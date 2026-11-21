import { useState } from 'react'
import TapHint from '@/components/tap-hint'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import { useI18n } from '@/app/i18n'
import { pieceCells, type Piece } from '@/app/lib/pieces'
import DraftToolbar from '@/app/games/[id]/_components/draft-toolbar'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import ProposeButton from '@/app/games/[id]/_components/propose-button'
import { useDraft } from '@/app/games/[id]/_viewmodels/use-draft'
import { useKeyboardLayout } from '@/app/games/[id]/_viewmodels/use-keyboard-layout'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from '../_lib/script'
import { isCorrect, pastTraces, TUTORIAL_DRAFT, type Tutorial } from '../_viewmodels/use-tutorial'
import TutorialNav from './tutorial-nav'

const cells = (piece: Piece) =>
  pieceCells(piece)
    .map(({ x, y, fill }) => `${x},${y},${fill}`)
    .toSorted()
    .join(' ')
/** By the cells covered, not the rotation: a Square looks the same however it is turned. */
const same = (a: Piece, b: Piece) =>
  a.shape === b.shape && a.colour === b.colour && cells(a) === cells(b)

const ALL_TRACES = pastTraces(TUTORIAL_STEPS.length)

/** The last two steps: the Draft of both pieces, then its Proposition. Mounted only then, storage being read. */
export default function TutorialDraft({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  const { keyboardLayout } = useKeyboardLayout()
  const draft = useDraft(TUTORIAL_DRAFT, keyboardLayout)
  const [wrong, setWrong] = useState<string | null>(null)
  const proposing = tutorial.current?.kind === 'propose'
  const current = JSON.stringify(draft.pieces)
  const complete = draft.pieces.length >= TUTORIAL_LAYOUT.pieces.length
  const missing = TUTORIAL_LAYOUT.pieces.find(
    (piece) => !draft.pieces.some((placed) => same(placed, piece))
  )

  const propose = () => {
    if (isCorrect(draft.pieces)) tutorial.finish()
    else setWrong(current)
  }

  return (
    <div
      className='mx-auto flex w-full flex-col gap-2'
      style={BOARD_COLUMN}
    >
      <DraftToolbar
        draft={draft}
        keyboardLayout={keyboardLayout}
      />
      <GameBoard
        paths={tutorial.paths}
        draft={draft}
        disabled
        highlighted={tutorial.highlighted}
        onFire={tutorial.fire}
        onHighlight={tutorial.setHighlighted}
        guide={proposing || !missing ? undefined : { x: missing.x, y: missing.y }}
      >
        {/* Two groups: the blink animates opacity and would override the dimming. */}
        <g opacity={0.35}>
          <g className='motion-safe:animate-[trace-blink_1.2s_ease-in-out_infinite]'>
            <TraceLines
              traces={ALL_TRACES}
              misses
            />
          </g>
        </g>
      </GameBoard>
      <TutorialNav
        tutorial={tutorial}
        nextDisabled={missing !== undefined}
      >
        {proposing ? (
          <span className='relative'>
            <ProposeButton
              disabled={!complete}
              pace={null}
              onPropose={propose}
            />
            {complete && wrong === null && <TapHint />}
          </span>
        ) : null}
      </TutorialNav>
      {wrong === current && (
        <p
          className='rounded-md border border-border px-3 py-2 text-sm font-medium'
          role='status'
          data-cy='tutorial-incorrect'
        >
          {t.tutorial.incorrect}
        </p>
      )}
    </div>
  )
}
