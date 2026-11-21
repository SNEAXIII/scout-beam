import { BEAM_SWATCHES, type BeamColour } from '@/app/lib/colours'
import { traceSegments } from '@/app/lib/trace'
import { TUTORIAL_LAYOUT } from '../_lib/script'

const SEGMENT_S = 0.35
const WIDTH = 0.1

// The grey of a miss and the outline of black: a fill alone would vanish on the dark board.
const stroke = (colour: BeamColour) => BEAM_SWATCHES[colour].outline ?? BEAM_SWATCHES[colour].fill

/** As on the rules page: each segment drawn in turn in the colour picked up so far, then the whole beam in its final colour. */
export default function DemoTrace({ entry }: Readonly<{ entry: number }>) {
  const traced = traceSegments(TUTORIAL_LAYOUT, entry)
  if (!traced) return null
  const { trace, colours } = traced
  const [first] = trace.points
  const final = stroke(trace.colour)
  const settle = { animation: `trace-show 0.25s ease-out ${colours.length * SEGMENT_S}s both` }
  return (
    <g
      fill='none'
      strokeLinecap='round'
      strokeLinejoin='round'
      className='demo-trace pointer-events-none'
      data-cy='demo-trace'
    >
      {colours.map((colour, i) => {
        const [a, b] = [trace.points[i], trace.points[i + 1]]
        return (
          <line
            key={i}
            x1={a.x}
            y1={a.y}
            x2={b.x}
            y2={b.y}
            stroke={stroke(colour)}
            strokeWidth={WIDTH}
            pathLength={1}
            strokeDasharray='1 1'
            style={{ animation: `trace-draw ${SEGMENT_S}s linear ${i * SEGMENT_S}s both` }}
          />
        )
      })}
      {trace.points.length === 1 ? (
        <circle
          cx={first.x}
          cy={first.y}
          r={0.18}
          fill={final}
          style={settle}
        />
      ) : (
        <polyline
          points={trace.points.map(({ x, y }) => `${x},${y}`).join(' ')}
          stroke={final}
          strokeWidth={WIDTH * 1.5}
          style={settle}
        />
      )}
    </g>
  )
}
