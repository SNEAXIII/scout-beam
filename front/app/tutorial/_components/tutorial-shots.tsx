import GhostPiece from '@/components/ghost-piece'
import { useI18n } from '@/app/i18n'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import { traceBeam } from '@/app/lib/trace'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from '../_lib/script'
import { useCandidateCycle } from '../_viewmodels/use-candidate-cycle'
import type { Tutorial } from '../_viewmodels/use-tutorial'
import TutorialCandidates from './tutorial-candidates'
import TutorialNav from './tutorial-nav'

const EMPTY = { pieces: [] }
const OPENER = TUTORIAL_STEPS.find((step) => step.kind === 'shot' && step.candidates)
// The candidates sit around the turn of the shot that opened them.
const TURN =
  OPENER?.kind === 'shot' ? traceBeam(TUTORIAL_LAYOUT, OPENER.entry)?.points[1] : undefined

/** A shot step: only the guided entry fires, its bare answer shows, then "Next" draws its Trace. */
export default function TutorialShots({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  const { current, fired, trace, phase, ghosts } = tutorial
  const remaining = ghosts.flatMap((ghost, i) => (ghost.eliminated ? [] : [String(i)]))
  // The shot that rules candidates out shows only those, crossed, so the player sees why.
  const ruledOut = fired && current?.kind === 'shot' ? (current.eliminated ?? []) : []
  const outIds = ruledOut.map((piece) => String(ghosts.findIndex((ghost) => ghost.piece === piece)))
  // When that shot hit something, the candidates left take turns too: the beam visibly bounces off them.
  const hit = outIds.length && tutorial.shot?.colour !== 'none' ? remaining : []
  const cycled = outIds.length ? [...outIds, ...hit] : remaining
  const shown = ghosts[Number(useCandidateCycle(cycled) ?? -1)]
  if (current?.kind !== 'shot') return null
  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
    >
      <div className='relative'>
        <GameBoard
          paths={tutorial.paths}
          layout={EMPTY}
          disabled={fired}
          highlighted={tutorial.highlighted}
          onFire={tutorial.fire}
          onHighlight={tutorial.setHighlighted}
          guide={fired ? undefined : current.entry}
        >
          {shown && (
            <GhostPiece
              // A fresh element per piece, so the cross never animates from one onto the next.
              key={ghosts.indexOf(shown)}
              piece={shown.piece}
              eliminated={shown.eliminated}
              highlighted
            />
          )}
          {TURN && remaining.length > 1 && (
            <g
              className='pointer-events-none'
              data-cy='tutorial-candidates-unknown'
              aria-hidden
            >
              <circle
                cx={TURN.x}
                cy={TURN.y}
                r={0.3}
                className='fill-foreground'
              />
              <text
                x={TURN.x}
                y={TURN.y}
                fontSize={0.4}
                fontWeight={700}
                textAnchor='middle'
                dominantBaseline='central'
                className='fill-background'
              >
                ?
              </text>
            </g>
          )}
          {phase === 'explain' && trace && (
            <TraceLines
              traces={[trace]}
              animated={trace.key}
              misses
            />
          )}
        </GameBoard>
        <TutorialCandidates
          ghosts={ghosts}
          shown={shown?.piece.shape}
        />
      </div>
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
