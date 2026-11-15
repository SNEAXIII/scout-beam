import { useCallback, useEffect, useState } from 'react'
import { getSession } from '@/app/services/auth'
import type { ServerMessage } from '@/app/services/game-socket'
import { getRoom } from '@/app/services/rooms'

/** Who may act again and when; a null actor is the caller, refused by the server. */
interface Priority {
  readonly actor: string | null
  readonly until: number
}

export interface RoomGame {
  readonly opponent: string | null
  readonly surrendered: boolean
  /** Unknown until the opponent's first shot since the page opened. */
  readonly opponentShots: number | null
  /** Milliseconds before a Duel lets the caller fire or propose again; 0 when they may. */
  readonly waitMs: number
}

/** A room game's opponent and, in a Duel, the caller's priority wait; idle outside a room. */
export function useRoomGame(roomCode: string | null, over: boolean) {
  const [me, setMe] = useState<string | null>(null)
  const [opponent, setOpponent] = useState<string | null>(null)
  const [surrendered, setSurrendered] = useState(false)
  const [opponentShots, setOpponentShots] = useState<number | null>(null)
  const [priority, setPriority] = useState<Priority | null>(null)
  const [now, setNow] = useState(() => Date.now())

  const hold = useCallback((ms: number) => {
    const at = Date.now()
    setNow(at)
    setPriority({ actor: null, until: at + ms })
  }, [])

  // One after the other: two 401s at once would race for the refresh cookie. Read again once over.
  useEffect(() => {
    if (!roomCode) return
    let live = true
    void (async () => {
      const { login } = await getSession()
      const room = await getRoom(roomCode)
      if (!live) return
      setMe(login)
      setOpponent((login === room.host ? room.opponent : room.host) ?? null)
      setSurrendered(room.surrendered ?? false)
      if (room.retry_after_ms) hold(room.retry_after_ms)
    })().catch(() => {
      // The game plays on without the opponent's login.
    })
    return () => {
      live = false
    }
  }, [roomCode, over, hold])

  const receive = (message: ServerMessage) => {
    if (message.type === 'opponent') setOpponentShots(message.shots)
    if (message.type !== 'priority') return
    // ponytail: `until` is the server's clock, a relative delay in the event if skew shows
    setNow(Date.now())
    setPriority({ actor: message.actor, until: Date.parse(message.until) })
  }

  // The opponent acting lifts the caller's wait: only the caller's own priority counts.
  const readyAt = priority && (priority.actor ?? me) === me ? priority.until : null
  useEffect(() => {
    if (readyAt === null) return
    const timer = setInterval(() => {
      const at = Date.now()
      setNow(at)
      if (at >= readyAt) clearInterval(timer)
    }, 250)
    return () => clearInterval(timer)
  }, [readyAt])

  const room: RoomGame = {
    opponent,
    surrendered,
    opponentShots,
    waitMs: readyAt === null ? 0 : Math.max(0, readyAt - now),
  }
  return { room, receive, hold }
}
