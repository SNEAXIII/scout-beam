import { useI18n } from '@/app/i18n'
import { BEAM_SWATCHES, PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, pieceCells, type Piece } from '@/app/lib/pieces'

const COLUMNS = 6
const ROWS = 5
const SPIKE: Piece = { shape: 'spike', colour: 'red', x: 1, y: 1, rotation: 0 }
const DIAMOND: Piece = { shape: 'diamond', colour: 'yellow', x: 2, y: 3, rotation: 0 }
const ENTRY = { x: 1.5, y: -0.55 }
const DRAW = ['beam-a', 'beam-b', 'beam-c'].map(
  (name) => `motion-safe:animate-[${name}_6s_linear_infinite]`
)

interface Example {
  readonly cy: string
  readonly pieces: readonly Piece[]
  /** The beam before, between and after its turns: none, red, then red + blue = violet. */
  readonly segments: readonly [string, string, string]
  readonly exit: { x: number; y: number }
}

// Both checked against `lib/trace.ts`, from entry 2 of the full grid: out by 24, and a Return.
const TURN: Example = {
  cy: 'beam-demo-turn',
  pieces: [SPIKE, { shape: 'spike', colour: 'blue', x: 4, y: 1, rotation: 2 }, DIAMOND],
  segments: ['M1.5 -0.25V1.5', 'M1.5 1.5H4.5', 'M4.5 1.5V5.25'],
  exit: { x: 4.5, y: 5.55 },
}
const RETURN: Example = {
  cy: 'beam-demo-return',
  pieces: [SPIKE, { shape: 'square', colour: 'blue', x: 4, y: 1, rotation: 0 }, DIAMOND],
  segments: ['M1.4 -0.25V1.4', 'M1.4 1.4H4', 'M4 1.4V1.6H1.6V-0.25'],
  exit: ENTRY,
}
const COLOURS = [BEAM_SWATCHES.none.outline, BEAM_SWATCHES.red.fill, BEAM_SWATCHES.violet.fill]

const lines = (count: number) => Array.from({ length: count + 1 }, (_, i) => i)

function BeamExample({ example, label }: { readonly example: Example; readonly label: string }) {
  return (
    <svg
      viewBox='-0.5 -1.1 7 7.2'
      className='w-full text-border'
      role='img'
      aria-label={label}
      data-cy={example.cy}
    >
      <rect
        width={COLUMNS}
        height={ROWS}
        className='fill-muted/40'
      />
      <g stroke='currentColor'>
        {lines(COLUMNS).map((x) => (
          <line
            key={`c${x}`}
            x1={x}
            x2={x}
            y2={ROWS}
            vectorEffect='non-scaling-stroke'
          />
        ))}
        {lines(ROWS).map((y) => (
          <line
            key={`r${y}`}
            y1={y}
            x2={COLUMNS}
            y2={y}
            vectorEffect='non-scaling-stroke'
          />
        ))}
      </g>
      {example.pieces.flatMap((piece) =>
        pieceCells(piece).map((cell) => (
          <polygon
            key={`${cell.x},${cell.y}`}
            points={cellPoints(cell)}
            fill={PIECE_SWATCHES[piece.colour].fill}
          />
        ))
      )}
      <circle
        cx={ENTRY.x}
        cy={ENTRY.y}
        r={0.3}
        fill='none'
        stroke={BEAM_SWATCHES.none.outline}
        strokeWidth={0.08}
      />
      <g
        fill='none'
        strokeWidth={0.12}
        strokeLinejoin='round'
        className='motion-safe:animate-[beam-fade_6s_linear_infinite]'
      >
        {example.segments.map((d, index) => (
          <path
            key={d}
            d={d}
            stroke={COLOURS[index]}
            pathLength={1}
            strokeDasharray='1 1'
            className={DRAW[index]}
          />
        ))}
        <circle
          cx={example.exit.x}
          cy={example.exit.y}
          r={0.22}
          fill={BEAM_SWATCHES.violet.fill}
          className='motion-safe:animate-[beam-exit_6s_linear_infinite]'
          data-cy={`${example.cy}-exit`}
        />
      </g>
    </svg>
  )
}

/** Two shots on a small grid: one leaves by another entry, the other comes back out, a Return. */
export default function BeamDemo() {
  const { t } = useI18n()
  const examples = [
    { example: TURN, label: t.landing.beamDemo.turn, caption: t.landing.beamDemo.turnCaption },
    {
      example: RETURN,
      label: t.landing.beamDemo.return,
      caption: t.landing.beamDemo.returnCaption,
    },
  ]
  return (
    <div
      className='grid w-full max-w-sm grid-cols-2 gap-3'
      data-cy='beam-demo'
    >
      {examples.map(({ example, label, caption }) => (
        <figure
          key={example.cy}
          className='flex flex-col items-center gap-1'
        >
          <BeamExample
            example={example}
            label={label}
          />
          <figcaption className='text-center text-sm text-muted-foreground'>{caption}</figcaption>
        </figure>
      ))}
    </div>
  )
}
