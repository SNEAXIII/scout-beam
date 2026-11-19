import { PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, pieceCells, type Piece } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'

interface GhostPieceProps {
  readonly piece: Piece
  readonly eliminated?: boolean
  readonly highlighted?: boolean
}

/** A piece as a translucent dashed outline, for inside a `GridSvg`; solid when highlighted, crossed out once eliminated (faded unless highlighted). */
export default function GhostPiece({
  piece,
  eliminated = false,
  highlighted = false,
}: GhostPieceProps) {
  const { fill } = PIECE_SWATCHES[piece.colour]
  const solid = highlighted && !eliminated
  const cells = pieceCells(piece)
  const left = Math.min(...cells.map((cell) => cell.x))
  const top = Math.min(...cells.map((cell) => cell.y))
  const right = Math.max(...cells.map((cell) => cell.x)) + 1
  const bottom = Math.max(...cells.map((cell) => cell.y)) + 1
  return (
    <g
      className='pointer-events-none'
      data-cy={`ghost-${piece.shape}`}
      data-eliminated={eliminated}
      data-highlighted={solid}
    >
      <g
        fill={fill}
        fillOpacity={solid ? 0.7 : 0.15}
        stroke={fill}
        strokeWidth={2}
        strokeDasharray={solid ? undefined : '4 3'}
        className={cn('motion-safe:transition-opacity', eliminated && !highlighted && 'opacity-30')}
      >
        {cells.map((cell) => (
          <polygon
            key={`${cell.x},${cell.y}`}
            points={cellPoints(cell)}
            vectorEffect='non-scaling-stroke'
          />
        ))}
      </g>
      <g
        stroke='currentColor'
        strokeWidth={0.08}
        strokeLinecap='round'
        className='text-foreground'
      >
        {[
          [left, right],
          [right, left],
        ].map(([x1, x2]) => (
          <line
            key={x1}
            x1={x1}
            y1={top}
            x2={x2}
            y2={bottom}
            pathLength={1}
            strokeDasharray='1 2'
            strokeDashoffset={eliminated ? 0 : 1.5}
            className='motion-safe:transition-[stroke-dashoffset] motion-safe:duration-500'
          />
        ))}
      </g>
    </g>
  )
}
