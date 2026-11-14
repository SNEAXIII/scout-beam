import { useCallback, useEffect, useEffectEvent, useRef, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import { ApiError, detailMessage } from '@/app/lib/api'
import { clearDraft } from '@/app/lib/draft'
import type { Piece } from '@/app/lib/pieces'
import { getAccessToken } from '@/app/lib/session'
import { openGameSocket, type GameSocket, type ServerMessage } from '@/app/services/game-socket'
import { getGame, type GameView, type PropositionVerdict } from '@/app/services/games'

const message = (caught: unknown, fallback: string) =>
  caught instanceof Error ? caught.message : fallback

interface PendingVerdict {
  resolve: (verdict: PropositionVerdict) => void
  reject: (error: ApiError) => void
}

/** A game over its socket: its shots, and the player's three actions on it. */
export function useGame(id: string) {
  const { t } = useI18n()
  const router = useRouter()
  const [game, setGame] = useState<GameView | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [connected, setConnected] = useState(false)
  const [surrendering, setSurrendering] = useState(false)
  const socket = useRef<GameSocket | null>(null)
  const inFlight = useRef(new Set<number>())
  const verdict = useRef<PendingVerdict | null>(null)

  const load = useCallback(async () => {
    try {
      setGame(await getGame(id))
    } catch (caught) {
      setError(message(caught, t.common.loadFailed))
    }
  }, [id, t])

  const refused = (status: number, detail: unknown, fallback: string) => {
    // 409: over meanwhile, in another tab — reload to show the layout.
    if (status === 409) void load()
    else setError(detailMessage(detail) ?? fallback)
  }

  const receive = useEffectEvent((received: ServerMessage) => {
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
        verdict.current?.resolve(received)
        verdict.current = null
        return
      case 'error':
        if (received.action === 'fire') inFlight.current.clear()
        if (received.action === 'surrender') setSurrendering(false)
        if (received.action === 'propose') {
          const text = detailMessage(received.detail) ?? t.draft.proposeFailed
          verdict.current?.reject(new ApiError(received.status, text))
          verdict.current = null
        } else if (received.action !== 'connect' || received.status !== 401) {
          const fallback =
            received.action === 'surrender' ? t.game.surrenderFailed : t.game.shotFailed
          refused(received.status, received.detail, fallback)
        }
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

  useEffect(() => {
    if (!getAccessToken()) {
      router.replace('/login')
      return
    }
    const opened = openGameSocket(id, { onMessage: receive, onConnected: connection })
    socket.current = opened
    return () => {
      opened.close()
      socket.current = null
    }
  }, [id, router])

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

  /** Rejects with the API's status: 409 when the game ended meanwhile, 422 naming a broken rule. */
  const propose = useCallback(
    (pieces: readonly Piece[]) =>
      new Promise<PropositionVerdict>((resolve, reject) => {
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

  return { game, error, connected, over, surrendering, load, fire, surrender, propose }
}
