'use client'

import { useMemo, useState } from 'react'
import { useParams } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import DraftPanel from './_components/draft-panel'
import GameBoard from './_components/game-board'
import GameHeader from './_components/game-header'
import ShotList from './_components/shot-list'
import { shotPaths } from './_components/shot-paths'
import { useGame } from './_viewmodels/use-game'

export default function GamePage() {
  const { id } = useParams<{ id: string }>()
  const { t } = useI18n()
  const { game, error, over, surrendering, load, fire, surrender } = useGame(id)
  const [highlighted, setHighlighted] = useState<string | null>(null)

  const shots = useMemo(() => game?.shots ?? [], [game])
  const paths = useMemo(() => shotPaths(shots), [shots])

  return (
    <main className='mx-auto flex w-full max-w-7xl flex-col gap-6 p-4 md:p-8'>
      <GameHeader
        status={game?.status ?? null}
        onSurrender={() => void surrender()}
        surrendering={surrendering}
      />
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='game-error'
        >
          {error}
        </p>
      )}
      {game === null && !error && (
        <p className='text-sm text-muted-foreground'>{t.common.loading}</p>
      )}
      {game && (
        <div
          className={cn(
            'grid items-start gap-6',
            over
              ? 'md:grid-cols-[minmax(0,1fr)_16rem]'
              : 'md:grid-cols-2 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_16rem]'
          )}
        >
          <div className='flex flex-col gap-3'>
            <GameBoard
              paths={paths}
              layout={game.layout}
              disabled={over}
              highlighted={highlighted}
              onFire={(entry) => void fire(entry)}
              onHighlight={setHighlighted}
            />
            {game.status !== 'in_progress' && (
              <p
                className='text-sm text-muted-foreground'
                data-cy={`game-${game.status}`}
              >
                {t.game[game.status]}
              </p>
            )}
          </div>
          {!over && (
            <DraftPanel
              gameId={id}
              onSolved={() => void load()}
            />
          )}
          <div className={cn(!over && 'md:col-span-2 xl:col-span-1')}>
            <ShotList
              shots={shots}
              paths={paths}
              highlighted={highlighted}
              onHighlight={setHighlighted}
            />
          </div>
        </div>
      )}
    </main>
  )
}
