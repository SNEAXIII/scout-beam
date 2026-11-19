import { ArrowRight } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import BeamMarker from '@/app/games/[id]/_components/beam-marker'
import type { ShotPath } from '@/app/games/[id]/_components/shot-paths'
import { BEAM_SWATCHES, PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, pieceCells, type Piece } from '@/app/lib/pieces'
import TapHint from '@/components/tap-hint'

const COLUMNS = 5
const ROWS = 4
const ENTRIES = Array.from({ length: 2 * (COLUMNS + ROWS) }, (_, i) => i + 1)
const VIEW = { x: -1, y: -1, width: COLUMNS + 2, height: ROWS + 2 }

const SPIKE: Piece = { shape: 'spike', colour: 'red', x: 1, y: 1, rotation: 0 }
// Down column 1 from entry 2, the spike's slanted side turns it east: out by entry 7, red.
const SHOT: ShotPath = { key: '2-7', label: 'A', colour: 'red', from: 2, to: 7 }

/** The centre of an entry, numbered like `lib/entries.ts` but around this small grid. */
function entryCentre(entry: number): { x: number; y: number } {
  const right = COLUMNS
  const bottom = right + ROWS
  const left = bottom + COLUMNS
  if (entry <= right) return { x: entry - 0.5, y: -0.5 }
  if (entry <= bottom) return { x: COLUMNS + 0.5, y: entry - right - 0.5 }
  if (entry <= left) return { x: left - entry + 0.5, y: ROWS + 0.5 }
  return { x: -0.5, y: ENTRIES.length - entry + 0.5 }
}

const percent = ({ x, y }: { x: number; y: number }) => ({
  left: `${((x - VIEW.x) / VIEW.width) * 100}%`,
  top: `${((y - VIEW.y) / VIEW.height) * 100}%`,
})

const SHOW_ENDS = 'motion-safe:animate-[shot-ends_5.4s_linear_infinite]'

/** A shot as the game answers it: the beam's path fades, only its two ends and colour remain. */
export default function ShotDemo() {
  const { t } = useI18n()
  const entry = entryCentre(SHOT.from)
  return (
    <figure
      className='flex w-full max-w-xs flex-col items-center gap-2'
      data-cy='shot-demo'
    >
      <div
        className='relative w-full'
        role='img'
        aria-label={t.landing.shotDemo}
      >
        <svg
          viewBox={`${VIEW.x} ${VIEW.y} ${VIEW.width} ${VIEW.height}`}
          className='block w-full text-border'
        >
          <rect
            width={COLUMNS}
            height={ROWS}
            className='fill-muted/40'
            stroke='currentColor'
            strokeWidth={0.04}
          />
          {ENTRIES.map((n) => {
            const { x, y } = entryCentre(n)
            return (
              <text
                key={n}
                x={x}
                y={y}
                fontSize={0.38}
                textAnchor='middle'
                dominantBaseline='central'
                className='fill-muted-foreground tabular-nums'
              >
                {n}
              </text>
            )
          })}
          <g
            fill='none'
            strokeWidth={0.12}
            strokeLinejoin='round'
            className='motion-safe:animate-[shot-path_5.4s_linear_infinite]'
          >
            {pieceCells(SPIKE).map((cell) => (
              <polygon
                key={`${cell.x},${cell.y}`}
                points={cellPoints(cell)}
                fill={PIECE_SWATCHES[SPIKE.colour].fill}
              />
            ))}
            <path
              d='M1.5 -0.25V1.5'
              stroke={BEAM_SWATCHES.none.outline}
              pathLength={1}
              strokeDasharray='1 1'
              className='motion-safe:animate-[shot-in_5.4s_linear_infinite]'
            />
            <path
              d='M1.5 1.5H5.25'
              stroke={BEAM_SWATCHES.red.fill}
              pathLength={1}
              strokeDasharray='1 1'
              className='motion-safe:animate-[shot-out_5.4s_linear_infinite]'
            />
          </g>
        </svg>
        {[SHOT.from, SHOT.to].map((end) => (
          <span
            key={end}
            className={`absolute -translate-1/2 ${SHOW_ENDS}`}
            style={percent(entryCentre(end))}
            data-cy={`shot-demo-end-${end}`}
          >
            <BeamMarker path={SHOT} />
          </span>
        ))}
        <TapHint
          className='motion-safe:animate-[shot-hint_5.4s_linear_infinite]'
          style={{
            ...percent({ x: entry.x - 0.5, y: entry.y - 0.5 }),
            right: 'auto',
            bottom: 'auto',
            width: `${100 / VIEW.width}%`,
            height: `${100 / VIEW.height}%`,
          }}
        />
      </div>
      <figcaption className={`flex flex-col items-center gap-1 text-sm ${SHOW_ENDS}`}>
        <span className='text-muted-foreground'>{t.landing.whatYouSee}</span>
        <span
          className='flex items-center gap-2'
          data-cy='shot-demo-answer'
        >
          <BeamMarker path={SHOT} />
          <span className='flex items-center gap-1.5 tabular-nums'>
            {SHOT.from}
            <ArrowRight className='size-3.5 text-muted-foreground' />
            {SHOT.to}
          </span>
          <span className='text-muted-foreground'>{t.colours[SHOT.colour]}</span>
        </span>
      </figcaption>
    </figure>
  )
}
