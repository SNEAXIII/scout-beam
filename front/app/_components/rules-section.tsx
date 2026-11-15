'use client'

import { useI18n } from '@/app/i18n'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { SHAPES, type PieceColour, type Shape } from '@/app/lib/pieces'
import ShapeIcon from '@/components/shape-icon'

const SHOWCASE: Record<Shape, PieceColour> = {
  square: 'red',
  bar: 'blue',
  spike: 'yellow',
  corner: 'white',
  diamond: 'red',
}

interface RuleProps {
  readonly text: string
  readonly cy: string
  /** An illustration drawn beside the paragraph. */
  readonly children?: React.ReactNode
}

function Rule({ text, cy, children }: RuleProps) {
  return (
    <li
      className='flex flex-col gap-3 md:flex-row md:items-center md:gap-6'
      data-cy={`rule-${cy}`}
    >
      <p className='flex-1 leading-relaxed'>{text}</p>
      {children}
    </li>
  )
}

export default function RulesSection() {
  const { t } = useI18n()
  const rules = t.landing.rules
  return (
    <section
      className='flex flex-col gap-6'
      data-cy='rules'
    >
      <h2 className='text-2xl font-semibold tracking-tight'>{t.landing.rulesTitle}</h2>
      <ol className='flex flex-col gap-6'>
        <Rule
          text={rules.grid}
          cy='grid'
        />
        <Rule
          text={rules.pieces}
          cy='pieces'
        >
          <ul
            className='flex items-center gap-3'
            aria-label={t.landing.piecesLabel}
          >
            {SHAPES.map((shape) => (
              <li
                key={shape}
                title={t.shapes[shape]}
              >
                <ShapeIcon
                  shape={shape}
                  rotation={0}
                  fill={PIECE_SWATCHES[SHOWCASE[shape]].fill}
                />
              </li>
            ))}
          </ul>
        </Rule>
        <Rule
          text={rules.shot}
          cy='shot'
        />
        <Rule
          text={rules.sides}
          cy='sides'
        />
        <Rule
          text={rules.colours}
          cy='colours'
        />
        <Rule
          text={rules.draft}
          cy='draft'
        />
        <Rule
          text={rules.surrender}
          cy='surrender'
        />
      </ol>
    </section>
  )
}
