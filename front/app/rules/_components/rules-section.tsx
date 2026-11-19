'use client'

import { EyeOff } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { SHAPES, type PieceColour, type Shape } from '@/app/lib/pieces'
import ShapeIcon from '@/components/shape-icon'
import BeamDemo from './beam-demo'
import ColourMixDemo from './colour-mix-demo'
import DeduceDemo from './deduce-demo'
import MarkersDemo from './markers-demo'
import ShotDemo from './shot-demo'

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
            {SHAPES.map((shape, index) => (
              <li
                key={shape}
                title={t.shapes[shape]}
                className='motion-safe:animate-[piece-appear_0.5s_ease-out_both]'
                style={{ animationDelay: `${index * 150}ms` }}
              >
                <ShapeIcon
                  shape={shape}
                  rotation={0}
                  fill={PIECE_SWATCHES[SHOWCASE[shape]].fill}
                  className='size-10'
                />
              </li>
            ))}
          </ul>
        </Rule>
        <Rule
          text={rules.shot}
          cy='shot'
        >
          <ShotDemo />
        </Rule>
        <li
          className='flex items-start gap-3 rounded-lg border bg-muted p-4'
          data-cy='rule-invisible'
        >
          <EyeOff className='mt-0.5 size-5 shrink-0 text-muted-foreground' />
          <p className='leading-relaxed'>{rules.invisible}</p>
        </li>
        <Rule
          text={rules.sides}
          cy='sides'
        >
          <BeamDemo />
        </Rule>
        <Rule
          text={rules.colours}
          cy='colours'
        >
          <ColourMixDemo label={t.landing.colourMix} />
        </Rule>
        <Rule
          text={rules.markers}
          cy='markers'
        >
          <MarkersDemo />
        </Rule>
        <Rule
          text={rules.deduce}
          cy='deduce'
        >
          <DeduceDemo />
        </Rule>
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
