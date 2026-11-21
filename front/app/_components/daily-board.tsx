'use client'

import { useMemo, useState } from 'react'
import Link from 'next/link'
import { ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useI18n } from '@/app/i18n'
import { FIRST_DAY } from '@/app/services/daily'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import { shotPaths } from '@/app/games/[id]/_components/shot-paths'
import type { useDaily } from '../_viewmodels/use-daily'

interface DailyBoardProps {
  readonly daily: ReturnType<typeof useDaily>
}

/** The picked day's grid: empty behind a Start button, else the caller's game on it. */
export default function DailyBoard({ daily }: DailyBoardProps) {
  const { t } = useI18n()
  const { today, date, setDate, game, starting, error, start } = daily
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const paths = useMemo(() => shotPaths(game?.shots ?? []), [game])

  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
    >
      <div className='flex flex-wrap items-center justify-between gap-2'>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.landing.title}</h1>
        <span className='text-lg font-medium text-muted-foreground'>{t.daily.title}</span>
        <Input
          type='date'
          aria-label={t.daily.pickDay}
          title={t.daily.pickDay}
          min={FIRST_DAY}
          max={today ?? undefined}
          value={date}
          onChange={(event) => setDate(event.target.value)}
          disabled={today === null}
          className='w-fit'
          data-cy='daily-day'
        />
      </div>
      {/* The ring of entries is symmetric: the board's centre is the grid's. */}
      <div className='relative'>
        <GameBoard
          paths={paths}
          layout={game?.layout}
          disabled
          highlighted={highlighted}
          onFire={() => undefined}
          onHighlight={setHighlighted}
        />
        {game === null && (
          <div className='absolute inset-0 flex items-center justify-center'>
            <Button
              size='lg'
              onClick={() => void start()}
              disabled={starting}
              data-cy='daily-start'
            >
              {t.daily.start}
            </Button>
          </div>
        )}
      </div>
      {game?.status === 'in_progress' && (
        <Button
          asChild
          className='w-fit'
        >
          <Link
            href={`/games/${game.id}`}
            data-cy='daily-resume'
          >
            {t.daily.resume}
            <ArrowRight />
          </Link>
        </Button>
      )}
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='daily-error'
        >
          {error}
        </p>
      )}
    </div>
  )
}
