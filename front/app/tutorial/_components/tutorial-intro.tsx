import { ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import TapHint from '@/components/tap-hint'
import { useI18n } from '@/app/i18n'
import { pieceCells } from '@/app/lib/pieces'
import { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import GridSvg from '@/app/games/[id]/_components/grid-svg'
import { INTRO_LAYOUT } from '../_lib/script'
import type { Tutorial } from '../_viewmodels/use-tutorial'

/** The intro step: the five shapes, each named under it, before the first shot. */
export default function TutorialIntro({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
    >
      <GridSvg
        layout={INTRO_LAYOUT}
        cy='tutorial-intro-grid'
        label={INTRO_LAYOUT.pieces.map((piece) => t.shapes[piece.shape]).join(', ')}
      >
        {INTRO_LAYOUT.pieces.map((piece) => {
          const cells = pieceCells(piece)
          const xs = cells.map((cell) => cell.x)
          const bottom = Math.max(...cells.map((cell) => cell.y)) + 1
          return (
            <text
              key={piece.shape}
              x={(Math.min(...xs) + Math.max(...xs) + 1) / 2}
              y={bottom + 0.6}
              fontSize={0.5}
              textAnchor='middle'
              className='fill-foreground capitalize'
              data-cy={`tutorial-intro-name-${piece.shape}`}
            >
              {t.shapes[piece.shape]}
            </text>
          )
        })}
      </GridSvg>
      <span className='relative ml-auto'>
        <Button
          size='sm'
          onClick={tutorial.next}
          data-cy='tutorial-start-shots'
        >
          {t.tutorial.begin}
          <ArrowRight />
        </Button>
        <TapHint />
      </span>
    </div>
  )
}
