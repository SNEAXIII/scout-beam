import { useEffect, useState } from 'react'
import { getBotStats, type BotStats } from '@/app/services/games'

/** The caller's Bot game stats; mounted for a registered player only. */
export function useBotStats() {
  const [stats, setStats] = useState<BotStats | null>(null)
  const [failed, setFailed] = useState(false)

  useEffect(() => {
    getBotStats().then(setStats, () => setFailed(true))
  }, [])

  return { stats, failed }
}
