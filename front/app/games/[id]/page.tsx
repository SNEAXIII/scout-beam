'use client'

import { useMemo, useState } from 'react'
import { useParams } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import DraftPanel from './_components/draft-panel'
import GameBoard, { BOARD_COLUMN } from './_components/game-board'
import GameHeader from './_components/game-header'
import ShotList from './_components/shot-list'
import { shotPaths } from './_components/shot-paths'
import { useGame } from './_viewmodels/use-game'

export default function GamePage() {
  const { id } = useParams<{ id: string }>()
  const { t } = useI18n()
  const { game, error, surrendering, load, fire, surrender } = useGame(id)
  const [highlighted, setHighlighted] = useState<string | null>(null)

  const shots = useMemo(() => game?.shots ?? [], [game])
  const paths = useMemo(() => shotPaths(shots), [shots])
  const board = {
    paths,
    highlighted,
    onFire: (entry: number) => void fire(entry),
    onHighlight: setHighlighted,
  }

  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-3 px-3 pb-3 md:px-4'>
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
        <div className='grid items-start gap-4 md:grid-cols-[minmax(0,1fr)_14rem]'>
          {game.status !== 'in_progress' ? (
            <div
              className='mx-auto flex w-full flex-col gap-3'
              style={BOARD_COLUMN}
            >
              <GameBoard
                {...board}
                layout={game.layout}
                disabled
              />
              <p
                className='text-sm text-muted-foreground'
                data-cy={`game-${game.status}`}
              >
                {t.game[game.status]}
              </p>
            </div>
          ) : (
            <DraftPanel
              {...board}
              gameId={id}
              onSolved={() => void load()}
            />
          )}
          <ShotList
            shots={shots}
            paths={paths}
            highlighted={highlighted}
            onHighlight={setHighlighted}
          />
        </div>
      )}
    </main>
  )
}
