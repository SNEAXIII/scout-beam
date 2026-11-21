import { useI18n } from '@/app/i18n'
import { BEAM_SWATCHES, PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, pieceCells, type Piece } from '@/app/lib/pieces'
import GhostPiece from '@/components/ghost-piece'

const COLUMNS = 5
const ROWS = 4
const SPIKE: Piece = { shape: 'spike', colour: 'yellow', x: 2, y: 1, rotation: 0 }
const CORNER: Piece = { shape: 'corner', colour: 'yellow', x: 2, y: 1, rotation: 0 }
const DIAMOND: Piece = { shape: 'diamond', colour: 'yellow', x: 1, y: 1, rotation: 0 }

// Checked against `lib/trace.ts` on a 5 × 4 grid, the Spike hidden: entry 3 turns out by 7 for all
// three; entries 4 and 2 miss straight down, where the Corner and the Diamond would turn them.
const SHOTS = [
  {
    d: 'M2.5 -0.25V1.5',
    stroke: BEAM_SWATCHES.none.outline,
    draw: 'motion-safe:animate-[deduce-in_9s_linear_infinite]',
  },
  {
    d: 'M2.5 1.5H5.25',
    stroke: BEAM_SWATCHES.yellow.fill,
    draw: 'motion-safe:animate-[deduce-out_9s_linear_infinite]',
  },
  {
    d: 'M3.5 -0.25V4.25',
    stroke: BEAM_SWATCHES.none.outline,
    draw: 'motion-safe:animate-[deduce-second_9s_linear_infinite]',
  },
  {
    d: 'M1.5 -0.25V4.25',
    stroke: BEAM_SWATCHES.none.outline,
    draw: 'motion-safe:animate-[deduce-third_9s_linear_infinite]',
  },
]
const RULED_OUT = [
  {
    piece: CORNER,
    gone: 'motion-safe:animate-[deduce-gone-a_9s_linear_infinite]',
    animate:
      'motion-safe:[&_g[fill]]:animate-[deduce-drop-a_9s_linear_infinite] motion-safe:[&_line]:animate-[deduce-strike-a_9s_linear_infinite]',
  },
  {
    piece: DIAMOND,
    gone: 'motion-safe:animate-[deduce-gone-b_9s_linear_infinite]',
    animate:
      'motion-safe:[&_g[fill]]:animate-[deduce-drop-b_9s_linear_infinite] motion-safe:[&_line]:animate-[deduce-strike-b_9s_linear_infinite]',
  },
]

const lines = (count: number) => Array.from({ length: count + 1 }, (_, i) => i)

/** One turn, three candidates for it; two crossing shots rule out all but the Spike. */
export default function DeduceDemo() {
  const { t } = useI18n()
  return (
    <svg
      viewBox='-0.5 -0.5 6 5'
      className='w-full max-w-xs text-board-line'
      role='img'
      aria-label={t.landing.deduceDemo}
      data-cy='deduce-demo'
    >
      <rect
        width={COLUMNS}
        height={ROWS}
        className='fill-board'
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
      <g className='motion-safe:animate-[deduce-loop_9s_linear_infinite]'>
        <g className='motion-safe:animate-[deduce-ghosts_9s_linear_infinite]'>
          <GhostPiece piece={SPIKE} />
          {/* Each ruled-out candidate is struck, then leaves: the overlap would hide the Spike. */}
          {RULED_OUT.map(({ piece, gone, animate }) => (
            <g
              key={piece.shape}
              className={gone}
            >
              <g
                className={animate}
                data-cy={`deduce-demo-out-${piece.shape}`}
              >
                <GhostPiece
                  piece={piece}
                  eliminated
                />
              </g>
            </g>
          ))}
        </g>
        <g
          fill={PIECE_SWATCHES[SPIKE.colour].fill}
          className='motion-safe:animate-[deduce-found_9s_linear_infinite]'
          data-cy='deduce-demo-found'
        >
          {pieceCells(SPIKE).map((cell) => (
            <polygon
              key={`${cell.x},${cell.y}`}
              points={cellPoints(cell)}
            />
          ))}
        </g>
        <g
          fill='none'
          strokeWidth={0.12}
          strokeLinejoin='round'
        >
          {SHOTS.map(({ d, stroke, draw }) => (
            <path
              key={d}
              d={d}
              stroke={stroke}
              pathLength={1}
              strokeDasharray='1 1'
              className={draw}
            />
          ))}
        </g>
      </g>
    </svg>
  )
}
