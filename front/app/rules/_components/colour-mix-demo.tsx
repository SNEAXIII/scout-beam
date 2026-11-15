import { BEAM_SWATCHES, PIECE_SWATCHES, type BeamColour } from '@/app/lib/colours'
import type { PieceColour } from '@/app/lib/pieces'

interface ColourMixDemoProps {
  readonly label: string
}

interface Mix {
  readonly inputs: readonly PieceColour[]
  readonly mix: BeamColour
  /** Fades the slot in after the previous one; every slot fades out together. */
  readonly show: string
  readonly delay: string
}

const MIXES: readonly Mix[] = [
  {
    inputs: ['red', 'blue'],
    mix: 'violet',
    show: 'motion-safe:animate-[mix-1_8s_linear_infinite]',
    delay: '0s',
  },
  {
    inputs: ['red', 'yellow'],
    mix: 'orange',
    show: 'motion-safe:animate-[mix-2_8s_linear_infinite]',
    delay: '1.6s',
  },
  {
    inputs: ['blue', 'yellow'],
    mix: 'green',
    show: 'motion-safe:animate-[mix-3_8s_linear_infinite]',
    delay: '3.2s',
  },
  {
    inputs: ['red', 'blue', 'yellow'],
    mix: 'black',
    show: 'motion-safe:animate-[mix-4_8s_linear_infinite]',
    delay: '4.8s',
  },
]

const SLOT = 10
const SPACING = 1.4
const SPREAD = 1.8

/** The beam colour each pair of red, blue and yellow mixes into, then all three. */
export default function ColourMixDemo({ label }: ColourMixDemoProps) {
  return (
    <svg
      viewBox={`0 0 ${SLOT * MIXES.length} 12`}
      className='w-full max-w-sm'
      role='img'
      aria-label={label}
      data-cy='colour-mix-demo'
    >
      {MIXES.map(({ inputs, mix, show, delay }, slot) => {
        const centre = SLOT * slot + SLOT / 2
        const middle = (inputs.length - 1) / 2
        return (
          <g
            key={mix}
            className={show}
            data-cy={`colour-mix-${mix}`}
          >
            {inputs.map((colour, index) => (
              <circle
                key={colour}
                cx={centre + (index - middle) * SPACING}
                cy={3.5}
                r={1.4}
                fill={PIECE_SWATCHES[colour].fill}
                fillOpacity={0.9}
                className='motion-safe:animate-[mix-join_8s_ease-out_infinite]'
                style={
                  {
                    '--spread': `${Math.sign(index - middle) * SPREAD}px`,
                    animationDelay: delay,
                  } as React.CSSProperties
                }
              />
            ))}
            <circle
              cx={centre}
              cy={8.5}
              r={2}
              fill={BEAM_SWATCHES[mix].fill}
              stroke={BEAM_SWATCHES[mix].outline ?? 'none'}
              strokeWidth={0.15}
              className='origin-center [transform-box:fill-box] motion-safe:animate-[mix-pop_8s_ease-out_infinite]'
              style={{ animationDelay: delay }}
            />
          </g>
        )
      })}
    </svg>
  )
}
