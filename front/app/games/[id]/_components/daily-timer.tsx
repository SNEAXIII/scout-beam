'use client'

import { useEffect, useState } from 'react'
import { Timer } from 'lucide-react'
import { fill, useI18n } from '@/app/i18n'
import { clock } from '@/app/lib/clock'

interface DailyTimerProps {
  readonly createdAt: string
  /** Freezes the timer. */
  readonly solvedAt: string | null
  readonly wrongPropositions: number
}

/** Time since the daily started, and its penalty: a minute per wrong proposition. */
export default function DailyTimer({ createdAt, solvedAt, wrongPropositions }: DailyTimerProps) {
  const { t } = useI18n()
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (solvedAt) return
    const timer = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(timer)
  }, [solvedAt])

  const end = solvedAt ? Date.parse(solvedAt) : now
  const seconds = Math.max(0, Math.floor((end - Date.parse(createdAt)) / 1000))
  return (
    <span
      className='flex items-center gap-1.5 font-mono text-sm tabular-nums'
      title={t.daily.timer}
      data-cy='daily-timer'
    >
      <Timer className='size-4 text-muted-foreground' />
      {clock(seconds)}
      {wrongPropositions > 0 && (
        <span
          className='text-muted-foreground'
          data-cy='daily-penalty'
        >
          {fill(t.daily.penalty, { count: wrongPropositions })}
        </span>
      )}
    </span>
  )
}
