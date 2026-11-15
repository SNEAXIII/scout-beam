import { useEffect, useEffectEvent, useRef, useState } from 'react'
import { useRouter } from 'next/navigation'
import { getSession } from '@/app/services/auth'
import { getRoom, joinRoom, type RoomView } from '@/app/services/rooms'

const POLL_MS = 2000

const toError = (caught: unknown) => (caught instanceof Error ? caught : new Error(String(caught)))

/** The room under `code`, the caller's login, and joining it; off to the game once it starts. */
export function useRoom(code: string) {
  const router = useRouter()
  const [room, setRoom] = useState<RoomView | null>(null)
  const [me, setMe] = useState<string | null>(null)
  const [joining, setJoining] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  // Bumped by `join`: a poll sent before it would answer the room as it was.
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
  useEffect(() => {
    if (status !== 'waiting' && status !== 'composing') return
    // ponytail: polling every 2 s, a room channel on a socket if it shows its limits
    let live = true
    let timer: ReturnType<typeof setTimeout> | undefined
    const next = () => {
      timer = setTimeout(() => void load().finally(() => live && next()), POLL_MS)
    }
    next()
    return () => {
      live = false
      clearTimeout(timer)
    }
  }, [status])

  const gameId = status === 'playing' || status === 'finished' ? room?.game_id : undefined
  useEffect(() => {
    if (gameId) router.replace(`/games/${gameId}`)
  }, [gameId, router])

  const join = async () => {
    setJoining(true)
    setError(null)
    generation.current += 1
    try {
      const seen = await joinRoom(code)
      generation.current += 1
      setRoom(seen)
    } catch (caught) {
      setError(toError(caught))
    } finally {
      setJoining(false)
    }
  }

  const member = room !== null && me !== null && (me === room.host || me === room.opponent)
  return { room, member, joining, error, join }
}
