import { useEffect, useEffectEvent, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { getToday, playDaily } from '@/app/services/daily'

/** Today's date from the API, the day picked to play, and starting its game. */
export function useDaily() {
  const { t } = useI18n()
  const router = useRouter()
  const [today, setToday] = useState<string | null>(null)
  const [date, setDate] = useState('')
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

  const play = async () => {
    setError(null)
    setStarting(true)
    try {
      const game = await playDaily(date)
      router.push(`/games/${game.id}`)
    } catch (caught) {
      setStarting(false)
      if (caught instanceof ApiError && caught.status === 404) setError(t.daily.noDaily)
      else setError(caught instanceof Error ? caught.message : t.daily.startFailed)
    }
  }

  return { today, date, setDate, starting, error, play }
}
