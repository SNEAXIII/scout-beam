import { BEAM_SWATCHES, PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, pieceCells, type Piece } from '@/app/lib/pieces'

interface BeamDemoProps {
  readonly label: string
}

const COLUMNS = 6
const ROWS = 5

const PIECES: readonly Piece[] = [
  { shape: 'spike', colour: 'red', x: 1, y: 1, rotation: 0 },
  { shape: 'square', colour: 'blue', x: 4, y: 1, rotation: 0 },
  { shape: 'diamond', colour: 'yellow', x: 2, y: 3, rotation: 0 },
]

// From the top of column 1: the spike's slanted side turns it east, the square sends it back,
// the spike turns it north again and it leaves by its own entry, red then red + blue = violet.
const SEGMENTS = [
  {
    d: 'M1.4 -0.25V1.4',
    colour: BEAM_SWATCHES.none.outline,
    animation: 'motion-safe:animate-[beam-a_6s_linear_infinite]',
  },
  {
    d: 'M1.4 1.4H4',
    colour: BEAM_SWATCHES.red.fill,
    animation: 'motion-safe:animate-[beam-b_6s_linear_infinite]',
  },
  {
    d: 'M4 1.4V1.6H1.6V-0.25',
    colour: BEAM_SWATCHES.violet.fill,
    animation: 'motion-safe:animate-[beam-c_6s_linear_infinite]',
  },
]

const lines = (count: number) => Array.from({ length: count + 1 }, (_, i) => i)

/** A shot on a small grid: one turn on a slanted side, one turn back on a straight side. */
export default function BeamDemo({ label }: BeamDemoProps) {
  return (
    <svg
      viewBox='-0.5 -1.1 7 6.6'
      className='w-full max-w-xs text-border'
      role='img'
      aria-label={label}
      data-cy='beam-demo'
    >
      <rect
        width={COLUMNS}
        height={ROWS}
        className='fill-muted/40'
      />
      {lines(COLUMNS).map((x) => (
        <line
          key={`c${x}`}
          x1={x}
          y1={0}
          x2={x}
          y2={ROWS}
          stroke='currentColor'
          vectorEffect='non-scaling-stroke'
        />
      ))}
      {lines(ROWS).map((y) => (
        <line
          key={`r${y}`}
          x1={0}
          y1={y}
          x2={COLUMNS}
          y2={y}
          stroke='currentColor'
          vectorEffect='non-scaling-stroke'
        />
      ))}
      {PIECES.map((piece) => (
        <g
          key={piece.shape}
          fill={PIECE_SWATCHES[piece.colour].fill}
        >
          {pieceCells(piece).map((cell) => (
            <polygon
              key={`${cell.x},${cell.y}`}
              points={cellPoints(cell)}
            />
          ))}
        </g>
      ))}
      <circle
        cx={1.5}
        cy={-0.55}
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
        {SEGMENTS.map(({ d, colour, animation }) => (
          <path
            key={d}
            d={d}
            stroke={colour}
            pathLength={1}
            strokeDasharray='1 1'
            className={animation}
          />
        ))}
        <circle
          cx={1.5}
          cy={-0.55}
          r={0.22}
          fill={BEAM_SWATCHES.violet.fill}
          className='motion-safe:animate-[beam-exit_6s_linear_infinite]'
          data-cy='beam-demo-exit'
        />
      </g>
    </svg>
  )
}
