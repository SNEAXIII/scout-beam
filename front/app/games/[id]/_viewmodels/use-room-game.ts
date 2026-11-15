import { useEffect, useState } from 'react'
import { getSession } from '@/app/services/auth'
import type { ServerMessage } from '@/app/services/game-socket'
import { getRoom } from '@/app/services/rooms'

export interface RoomGame {
  readonly opponent: string | null
  readonly surrendered: boolean
  /** Unknown until the room is read; live `opponent` events keep it current. */
  readonly opponentShots: number | null
}

/** A room game's opponent; idle outside a room. */
export function useRoomGame(roomCode: string | null, over: boolean) {
  const [opponent, setOpponent] = useState<string | null>(null)
  const [surrendered, setSurrendered] = useState(false)
  const [opponentShots, setOpponentShots] = useState<number | null>(null)

  // One after the other: two 401s at once would race for the refresh cookie. Read again once over.
  useEffect(() => {
    if (!roomCode) return
    let live = true
    void (async () => {
      const { login } = await getSession()
      const room = await getRoom(roomCode)
      if (!live) return
      setOpponent((login === room.host ? room.opponent : room.host) ?? null)
      setSurrendered(room.surrendered ?? false)
      const seeded = room.opponent_shots
      // A count only grows: a live event may land before this read does.
      if (seeded !== undefined) setOpponentShots((known) => Math.max(known ?? 0, seeded))
    })().catch(() => {
      // The game plays on without the opponent's login.
    })
    return () => {
      live = false
    }
  }, [roomCode, over])

  const receive = (message: ServerMessage) => {
    if (message.type === 'opponent') setOpponentShots(message.shots)
  }

  const room: RoomGame = { opponent, surrendered, opponentShots }
  return { room, receive }
}
