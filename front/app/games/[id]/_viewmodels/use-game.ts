import { useCallback, useEffect, useEffectEvent, useRef, useState } from 'react'
import { useI18n } from '@/app/i18n'
import { ApiError, detailMessage } from '@/app/lib/api'
import { clearDraft } from '@/app/lib/draft'
import type { Piece } from '@/app/lib/pieces'
import { openGameSocket, type GameSocket, type ServerMessage } from '@/app/services/game-socket'
import { getGame, type GameView, type PropositionVerdict } from '@/app/services/games'
import { usePace } from './use-pace'
import { useRoomGame } from './use-room-game'

const message = (caught: unknown, fallback: string) =>
  caught instanceof Error ? caught.message : fallback

interface PendingVerdict {
  /** Null when refused as too early. */
  resolve: (verdict: PropositionVerdict | null) => void
  reject: (error: ApiError) => void
}

/** A game over its socket: its shots, and the player's three actions on it. */
export function useGame(id: string) {
  const { t } = useI18n()
  const [game, setGame] = useState<GameView | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [connected, setConnected] = useState(false)
  const [surrendering, setSurrendering] = useState(false)
  const socket = useRef<GameSocket | null>(null)
  const inFlight = useRef(new Set<number>())
  const verdict = useRef<PendingVerdict | null>(null)
  const { room, receive: receiveRoom } = useRoomGame(
    game?.room_code ?? null,
    game !== null && game.status !== 'in_progress'
  )
  const { pace, receive: receivePace, refused } = usePace(game)

  const load = useCallback(async () => {
    try {
      setGame(await getGame(id))
    } catch (caught) {
      setError(message(caught, t.common.loadFailed))
    }
  }, [id, t])

  const failed = (received: Extract<ServerMessage, { type: 'error' }>) => {
    const wait = received.retry_after_ms
    if (wait !== undefined) refused(received.action, wait)
    if (received.action === 'fire') inFlight.current.clear()
    if (received.action === 'surrender') setSurrendering(false)
    if (received.action === 'propose') {
      const text = detailMessage(received.detail) ?? t.draft.proposeFailed
      // Too early is no failure: the cooldown, resynced, shows on the button.
      if (wait === undefined) verdict.current?.reject(new ApiError(received.status, text))
      else verdict.current?.resolve(null)
      verdict.current = null
      return
    }
    if (wait !== undefined || (received.action === 'connect' && received.status === 401)) return
    // 409: over meanwhile, in another tab — reload to show the layout.
    if (received.status === 409) return void load()
    const fallback = received.action === 'surrender' ? t.game.surrenderFailed : t.game.shotFailed
    setError(detailMessage(received.detail) ?? fallback)
  }

  const receive = useEffectEvent((received: ServerMessage) => {
    receiveRoom(received)
    receivePace(received)
    switch (received.type) {
      case 'game':
        setGame(received.game)
        setSurrendering(false)
        return
      case 'shot':
        inFlight.current.delete(received.shot.entry)
        setGame((current) =>
          current && !current.shots.some((known) => known.entry === received.shot.entry)
            ? { ...current, shots: [...current.shots, received.shot] }
            : current
        )
        return
      case 'verdict':
        // A wrong verdict comes with no snapshot: counted here, for a daily's penalty.
        if (!received.correct)
          setGame(
            (current) =>
              current && { ...current, wrong_propositions: current.wrong_propositions + 1 }
          )
        verdict.current?.resolve(received)
        verdict.current = null
        return
      case 'error':
        failed(received)
    }
  })

  const connection = useEffectEvent((up: boolean) => {
    setConnected(up)
    if (up) return setError(null)
    inFlight.current.clear()
    setSurrendering(false)
    verdict.current?.reject(new ApiError(0, t.game.offline))
    verdict.current = null
  })

  // Signed out, the socket is closed with 4401: it renews the session, else goes to /login.
  useEffect(() => {
    const opened = openGameSocket(id, { onMessage: receive, onConnected: connection })
    socket.current = opened
    return () => {
      opened.close()
      socket.current = null
    }
  }, [id])

  const fire = useCallback(
    (entry: number) => {
      if (inFlight.current.has(entry)) return
      setError(null)
      if (!socket.current?.send({ type: 'fire', entry })) return setError(t.game.offline)
      inFlight.current.add(entry)
    },
    [t]
  )

  const surrender = useCallback(() => {
    setError(null)
    if (!socket.current?.send({ type: 'surrender' })) return setError(t.game.offline)
    setSurrendering(true)
  }, [t])

  /** Null when too early; rejects with the API's status: 409 when over meanwhile, 422 naming a rule. */
  const propose = useCallback(
    (pieces: readonly Piece[]) =>
      new Promise<PropositionVerdict | null>((resolve, reject) => {
        if (!socket.current?.send({ type: 'propose', pieces })) {
          reject(new ApiError(0, t.game.offline))
          return
        }
        verdict.current = { resolve, reject }
      }),
    [t]
  )

  const over = game !== null && game.status !== 'in_progress'

  useEffect(() => {
    if (over) clearDraft(id)
  }, [over, id])

  return { game, room, pace, error, connected, over, surrendering, load, fire, surrender, propose }
}
