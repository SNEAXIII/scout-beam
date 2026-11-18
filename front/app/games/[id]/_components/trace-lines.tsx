import { useId } from 'react'
import { BEAM_SWATCHES } from '@/app/lib/colours'
import { GRID_COLUMNS, GRID_ROWS } from '@/app/lib/pieces'
import type { KeyedTrace } from '../_viewmodels/use-traces'

interface TraceLinesProps {
  /** Drawn in order, the last on top; those of colour `none` touched no piece and are skipped. */
  readonly traces: readonly KeyedTrace[]
  /** The trace drawn from its entry to its exit, again each time it changes. */
  readonly animated?: string | null
}

/** In screen pixels: the strokes do not scale with the grid. */
const WIDTH = 3
const OUTLINE_WIDTH = 6
/** In grid units; the reveal is wider than any stroke it uncovers. */
const DISC_RADIUS = 0.18
const REVEAL_WIDTH = 0.5

function TraceShape({ trace, animated }: { trace: KeyedTrace; animated: boolean }) {
  const mask = `trace-${useId().replaceAll(/[^\w-]/g, '')}`
  const { fill, outline } = BEAM_SWATCHES[trace.colour]
  const [first] = trace.points
  // Bounced straight back in its first cell: a disc, ringed to stand out on the piece it hit.
  if (trace.points.length === 1)
    return (
      <circle
        cx={first.x}
        cy={first.y}
        r={DISC_RADIUS}
        fill={fill}
        className={outline ? undefined : 'stroke-background'}
        stroke={outline}
        strokeWidth={WIDTH / 2}
        vectorEffect='non-scaling-stroke'
      />
    )
  const points = trace.points.map(({ x, y }) => `${x},${y}`).join(' ')
  const line = (stroke: string, width: number) => (
    <polyline
      points={points}
      stroke={stroke}
      strokeWidth={width}
      vectorEffect='non-scaling-stroke'
    />
  )
  return (
    <>
      {/* Dashes of a non-scaling stroke ignore `pathLength`: a mask in grid units draws it instead. */}
      {animated && (
        <mask
          id={mask}
          maskUnits='userSpaceOnUse'
          x={-1}
          y={-1}
          width={GRID_COLUMNS + 2}
          height={GRID_ROWS + 2}
        >
          <polyline
            points={points}
            stroke='white'
            strokeWidth={REVEAL_WIDTH}
            pathLength={1}
            strokeDasharray='1 1'
            className='motion-safe:animate-[trace-draw_0.8s_ease-out]'
          />
        </mask>
      )}
      <g mask={animated ? `url(#${mask})` : undefined}>
        {outline && line(outline, OUTLINE_WIDTH)}
        {line(fill, WIDTH)}
      </g>
    </>
  )
}

/** Beams through the revealed layout, each in its final colour. */
export default function TraceLines({ traces, animated = null }: TraceLinesProps) {
  return (
    <g
      fill='none'
      strokeLinejoin='round'
      className='pointer-events-none'
    >
      {traces
        .filter((trace) => trace.colour !== 'none')
        .map((trace) => (
          <g
            key={trace.key}
            data-cy='trace'
            data-entry={trace.entry}
            data-exit={trace.exit}
            data-colour={trace.colour}
          >
            <TraceShape
              trace={trace}
              animated={trace.key === animated}
            />
          </g>
        ))}
    </g>
  )
}
