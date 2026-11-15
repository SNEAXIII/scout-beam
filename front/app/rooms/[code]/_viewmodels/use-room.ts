import { useEffect, useEffectEvent, useRef, useState } from 'react'
import { useRouter } from 'next/navigation'
import { getSession } from '@/app/services/auth'
import { openRoomSocket } from '@/app/services/game-socket'
import { getRoom, joinRoom, type RoomView } from '@/app/services/rooms'

const toError = (caught: unknown) => (caught instanceof Error ? caught : new Error(String(caught)))

/** The room under `code`, the caller's login, and joining it; off to the game once it starts. */
export function useRoom(code: string) {
  const router = useRouter()
  const [room, setRoom] = useState<RoomView | null>(null)
  const [me, setMe] = useState<string | null>(null)
  const [joining, setJoining] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  // Bumped by `join`: a read sent before it would answer the room as it was.
  const generation = useRef(0)

  const load = useEffectEvent(() => {
    const sent = generation.current
    return getRoom(code).then(
      (seen) => {
        if (sent !== generation.current) return
        setError(null)
        setRoom(seen)
      },
      (caught: unknown) => sent === generation.current && setError(toError(caught))
    )
  })

  // One after the other: two 401s at once would race for the refresh cookie.
  useEffect(() => {
    getSession().then(
      (profile) => {
        setMe(profile.login)
        return load()
      },
      (caught: unknown) => setError(toError(caught))
    )
  }, [code])

  const status = room?.status
  const lobby = status === 'waiting' || status === 'composing'
  useEffect(() => {
    if (!lobby) return
    const socket = openRoomSocket(code, () => void load())
    return socket.close
  }, [lobby, code])

  const gameId = status === 'playing' || status === 'finished' ? room?.game_id : undefined
  useEffect(() => {
    if (gameId) router.replace(`/games/${gameId}`)
  }, [gameId, router])

  /** The room an action answered; a read sent before it would answer the room as it was. */
  const update = (seen: RoomView) => {
    generation.current += 1
    setRoom(seen)
  }

  const join = async () => {
    setJoining(true)
    setError(null)
    generation.current += 1
    try {
      update(await joinRoom(code))
    } catch (caught) {
      setError(toError(caught))
    } finally {
      setJoining(false)
    }
  }

  const member = room !== null && me !== null && (me === room.host || me === room.opponent)
  return { room, member, joining, error, join, update }
}
