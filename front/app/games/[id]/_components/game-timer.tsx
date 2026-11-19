'use client'

import { useEffect, useState } from 'react'
import { Timer } from 'lucide-react'
import { fill, useI18n } from '@/app/i18n'
import { clock } from '@/app/lib/clock'

interface GameTimerProps {
  readonly createdAt: string
  /** Freezes the timer. */
  readonly endedAt: string | null
  readonly wrongPropositions: number
}

/** Time since the game started, and its penalty: a minute per wrong proposition. */
export default function GameTimer({ createdAt, endedAt, wrongPropositions }: GameTimerProps) {
  const { t } = useI18n()
  const [now, setNow] = useState(() => Date.now())

  useEffect(() => {
    if (endedAt) return
    const timer = setInterval(() => setNow(Date.now()), 1000)
    return () => clearInterval(timer)
  }, [endedAt])

  const end = endedAt ? Date.parse(endedAt) : now
  const seconds = Math.max(0, Math.floor((end - Date.parse(createdAt)) / 1000))
  return (
    <span
      className='flex items-center gap-1.5 font-mono text-sm tabular-nums'
      title={t.game.timer}
      data-cy='game-timer'
    >
      <Timer className='size-4 text-muted-foreground' />
      {clock(seconds)}
      {wrongPropositions > 0 && (
        <span
          className='text-muted-foreground'
          data-cy='game-penalty'
        >
          {fill(t.daily.penalty, { count: wrongPropositions })}
        </span>
      )}
    </span>
  )
}
