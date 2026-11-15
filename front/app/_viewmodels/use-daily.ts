import { useEffect, useEffectEvent, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { findDaily, getToday, playDaily } from '@/app/services/daily'
import type { GameView } from '@/app/services/games'

interface Found {
  readonly date: string
  readonly game: GameView | null
}

/** Today's date from the API, the day picked, the caller's game of it, and the shot starting it. */
export function useDaily() {
  const { t } = useI18n()
  const router = useRouter()
  const [today, setToday] = useState<string | null>(null)
  const [date, setDate] = useState('')
  const [found, setFound] = useState<Found | null>(null)
  const [starting, setStarting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const failed = useEffectEvent((caught: unknown) =>
    setError(caught instanceof Error ? caught.message : t.common.loadFailed)
  )

  useEffect(() => {
    getToday().then((day) => {
      setToday(day)
      setDate(day)
    }, failed)
  }, [])

  useEffect(() => {
    if (!date) return
    let current = true
    findDaily(date).then((game) => current && setFound({ date, game }), failed)
    return () => {
      current = false
    }
  }, [date])

  /** Starts the day's game, then lets the game page fire this first shot. */
  const start = async (entry: number) => {
    setError(null)
    setStarting(true)
    try {
      const game = await playDaily(date)
      router.push(`/games/${game.id}?fire=${entry}`)
    } catch (caught) {
      setStarting(false)
      if (caught instanceof ApiError && caught.status === 404) setError(t.daily.noDaily)
      else setError(caught instanceof Error ? caught.message : t.daily.startFailed)
    }
  }

  // undefined while the picked day's game is being looked up.
  const game = found?.date === date ? found.game : undefined
  return { today, date, setDate, game, starting, error, start }
}
