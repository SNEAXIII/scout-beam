import { PIECE_SWATCHES } from '@/app/lib/colours'
import { cellPoints, GRID_COLUMNS, GRID_ROWS, pieceCells, type Layout } from '@/app/lib/pieces'

interface GridSvgProps {
  readonly layout?: Layout
  readonly style?: React.CSSProperties
}

const COLUMN_LINES = Array.from({ length: GRID_COLUMNS + 1 }, (_, i) => i)
const ROW_LINES = Array.from({ length: GRID_ROWS + 1 }, (_, i) => i)

/** The grid, one unit per cell, and the hidden layout's pieces once revealed. */
export default function GridSvg({ layout, style }: GridSvgProps) {
  return (
    <svg
      viewBox={`0 0 ${GRID_COLUMNS} ${GRID_ROWS}`}
      className='size-full rounded-sm bg-muted/40 text-border'
      style={style}
      role='img'
      aria-label={layout ? 'Grille et pièces cachées' : 'Grille'}
      data-cy='game-grid'
    >
      {COLUMN_LINES.map((x) => (
        <line
          key={`c${x}`}
          x1={x}
          y1={0}
          x2={x}
          y2={GRID_ROWS}
          stroke='currentColor'
          vectorEffect='non-scaling-stroke'
        />
      ))}
      {ROW_LINES.map((y) => (
        <line
          key={`r${y}`}
          x1={0}
          y1={y}
          x2={GRID_COLUMNS}
          y2={y}
          stroke='currentColor'
          vectorEffect='non-scaling-stroke'
        />
      ))}
      {layout?.pieces.map((piece) => (
        <g
          key={piece.shape}
          fill={PIECE_SWATCHES[piece.colour].fill}
          data-cy={`piece-${piece.shape}`}
          data-colour={piece.colour}
        >
          {pieceCells(piece).map((cell) => (
            <polygon
              key={`${cell.x},${cell.y}`}
              points={cellPoints(cell)}
            />
          ))}
        </g>
      ))}
    </svg>
  )
}
