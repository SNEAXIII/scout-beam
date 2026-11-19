import { ArrowRight, Eye, EyeOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import GhostPiece from '@/components/ghost-piece'
import { useI18n } from '@/app/i18n'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import type { Tutorial } from '../_viewmodels/use-tutorial'

const EMPTY = { pieces: [] }

/** A shot step: only the guided entry fires, then its answer can be traced before moving on. */
export default function TutorialShots({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  const { current, fired, trace, traceShown } = tutorial
  if (current?.kind !== 'shot') return null
  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
    >
      <GameBoard
        paths={tutorial.paths}
        layout={EMPTY}
        disabled={fired}
        highlighted={tutorial.highlighted}
        onFire={tutorial.fire}
        onHighlight={tutorial.setHighlighted}
        guide={fired ? undefined : current.entry}
      >
        {tutorial.ghosts.map(({ piece, eliminated }) => (
          <GhostPiece
            key={piece.shape}
            piece={piece}
            eliminated={eliminated}
          />
        ))}
        {traceShown && trace && (
          <TraceLines
            traces={[trace]}
            animated={trace.key}
            misses
          />
        )}
      </GameBoard>
      <p
        className='text-sm text-muted-foreground'
        data-cy='tutorial-fire-first'
      >
        {t.tutorial.fireFirst}
      </p>
      {fired && (
        <div className='flex flex-wrap items-center gap-2'>
          {trace && (
            <Button
              size='sm'
              variant='outline'
              onClick={tutorial.toggleTrace}
              data-cy='tutorial-see-trace'
            >
              {traceShown ? <EyeOff /> : <Eye />}
              {traceShown ? t.tutorial.hideTrace : t.tutorial.seeTrace}
            </Button>
          )}
          <Button
            size='sm'
            className='ml-auto'
            onClick={tutorial.next}
            data-cy='tutorial-next'
          >
            {t.tutorial.next}
            <ArrowRight />
          </Button>
        </div>
      )}
    </div>
  )
}
