import { PIECE_SWATCHES } from '@/app/lib/colours'
import {
  cellPoints,
  GRID_COLUMNS,
  GRID_ROWS,
  pieceCells,
  SHAPE_NAMES,
  type Layout,
  type Piece,
  type Shape,
} from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'

interface GridSvgProps {
  readonly layout?: Layout
  readonly style?: React.CSSProperties
  readonly className?: string
  /** `data-cy` of the grid. */
  readonly cy?: string
  readonly label?: string
  /** Pieces are `data-cy="{piecePrefix}-{shape}"`. */
  readonly piecePrefix?: string
  /** Drawn translucent, outlined when it does not fit. */
  readonly preview?: { readonly piece: Piece; readonly fits: boolean }
  readonly onPieceClick?: (shape: Shape) => void
}

const COLUMN_LINES = Array.from({ length: GRID_COLUMNS + 1 }, (_, i) => i)
const ROW_LINES = Array.from({ length: GRID_ROWS + 1 }, (_, i) => i)

const cells = (piece: Piece, outlined = false) =>
  pieceCells(piece).map((cell) => (
    <polygon
      key={`${cell.x},${cell.y}`}
      points={cellPoints(cell)}
      vectorEffect={outlined ? 'non-scaling-stroke' : undefined}
    />
  ))

/** The grid, one unit per cell, and a layout's pieces: the hidden one once revealed, or a draft's. */
export default function GridSvg({
  layout,
  style,
  className,
  cy = 'game-grid',
  label,
  piecePrefix = 'piece',
  preview,
  onPieceClick,
}: GridSvgProps) {
  return (
    <svg
      viewBox={`0 0 ${GRID_COLUMNS} ${GRID_ROWS}`}
      className={cn('size-full rounded-sm bg-muted/40 text-border', className)}
      style={style}
      role='img'
      aria-label={label ?? (layout ? 'Grille et pièces cachées' : 'Grille')}
      data-cy={cy}
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
          className={cn(onPieceClick && 'pointer-events-auto cursor-pointer hover:opacity-70')}
          onClick={onPieceClick && (() => onPieceClick(piece.shape))}
          data-cy={`${piecePrefix}-${piece.shape}`}
          data-colour={piece.colour}
        >
          {onPieceClick && <title>{`Retirer la pièce « ${SHAPE_NAMES[piece.shape]} »`}</title>}
          {cells(piece)}
        </g>
      ))}
      {preview && (
        <g
          fill={PIECE_SWATCHES[preview.piece.colour].fill}
          fillOpacity={preview.fits ? 0.5 : 0.2}
          className='text-foreground'
          stroke={preview.fits ? 'none' : 'currentColor'}
          strokeWidth={2}
          strokeDasharray='4 3'
          data-cy={`${piecePrefix}-preview`}
          data-fits={preview.fits}
        >
          {cells(preview.piece, !preview.fits)}
        </g>
      )}
    </svg>
  )
}
