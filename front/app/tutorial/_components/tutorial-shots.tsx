import GhostPiece from '@/components/ghost-piece'
import { useI18n } from '@/app/i18n'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import type { Tutorial } from '../_viewmodels/use-tutorial'
import TutorialNav from './tutorial-nav'

const EMPTY = { pieces: [] }

/** A shot step: only the guided entry fires, its bare answer shows, then "Next" draws its Trace. */
export default function TutorialShots({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  const { current, fired, trace, phase } = tutorial
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
        {phase === 'explain' && trace && (
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
      <TutorialNav
        tutorial={tutorial}
        nextDisabled={!fired}
        hint={fired}
      />
    </div>
  )
}
