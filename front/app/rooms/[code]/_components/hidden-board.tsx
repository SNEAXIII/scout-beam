'use client'

import { useMemo } from 'react'
import { LoaderCircle } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import type { Piece } from '@/app/lib/pieces'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import { shotPaths } from '@/app/games/[id]/_components/shot-paths'

const noop = () => {}

/** The layout just hidden, frozen under the wait for the opponent's. */
export default function HiddenBoard({ pieces }: Readonly<{ pieces: readonly Piece[] }>) {
  const { t } = useI18n()
  const paths = useMemo(() => shotPaths([]), [])
  const layout = useMemo(() => ({ pieces: [...pieces] }), [pieces])
  return (
    <div
      className='relative mx-auto w-full'
      style={BOARD_COLUMN}
      data-cy='room-state-hidden'
    >
      <GameBoard
        paths={paths}
        layout={layout}
        disabled
        highlighted={null}
        onFire={noop}
        onHighlight={noop}
        hideEntries
      />
      <p
        className='absolute inset-0 m-auto flex h-fit w-fit max-w-[80%] items-center gap-3 rounded-lg bg-card/90 px-5 py-3 text-center text-lg font-semibold'
        role='status'
      >
        <LoaderCircle className='size-5 shrink-0 animate-spin text-muted-foreground' />
        {t.rooms.hidden}
      </p>
    </div>
  )
}
