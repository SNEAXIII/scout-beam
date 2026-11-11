import { useMemo, useState } from 'react'
import {
  fitsGrid,
  GRID_COLUMNS,
  GRID_ROWS,
  pieceCells,
  SHAPE_NAMES,
  type Shape,
} from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'
import GridSvg from './grid-svg'
import type { Draft } from './use-draft'

interface DraftBoardProps {
  readonly draft: Draft
}

const CELLS = Array.from({ length: GRID_ROWS }, (_, y) =>
  Array.from({ length: GRID_COLUMNS }, (_, x) => ({ x, y }))
).flat()

/** The draft grid: a click places the picked shape with its top-left there, or removes a piece. */
export default function DraftBoard({ draft }: DraftBoardProps) {
  const [hovered, setHovered] = useState<{ x: number; y: number } | null>(null)
  const owners = useMemo(() => {
    const owner = new Map<string, Shape>()
    for (const piece of draft.pieces)
      for (const cell of pieceCells(piece)) owner.set(`${cell.x},${cell.y}`, piece.shape)
    return owner
  }, [draft.pieces])

  const previewPiece = hovered && draft.pieceAt(hovered.x, hovered.y)
  const preview =
    previewPiece && !owners.has(`${hovered.x},${hovered.y}`)
      ? { piece: previewPiece, fits: fitsGrid(previewPiece) }
      : undefined

  const describe = (x: number, y: number, owner: Shape | undefined) => {
    if (owner) return `Case ${x + 1}, ${y + 1} : retirer la pièce « ${SHAPE_NAMES[owner]} »`
    if (!draft.shape) return `Case ${x + 1}, ${y + 1}`
    return `Case ${x + 1}, ${y + 1} : placer la pièce « ${SHAPE_NAMES[draft.shape]} »`
  }

  return (
    <div
      className='relative w-full'
      style={{ aspectRatio: `${GRID_COLUMNS} / ${GRID_ROWS}` }}
      data-cy='draft-board'
    >
      <div
        className='grid size-full rounded-sm bg-muted/40'
        style={{
          gridTemplateColumns: `repeat(${GRID_COLUMNS}, minmax(0, 1fr))`,
          gridTemplateRows: `repeat(${GRID_ROWS}, minmax(0, 1fr))`,
        }}
      >
        {CELLS.map(({ x, y }) => {
          const owner = owners.get(`${x},${y}`)
          return (
            <button
              key={`${x},${y}`}
              type='button'
              className={cn(
                'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring',
                !owner && draft.shape ? 'hover:bg-accent/60' : !owner && 'cursor-default'
              )}
              onClick={() => (owner ? draft.remove(owner) : draft.place(x, y))}
              onMouseEnter={() => setHovered({ x, y })}
              onMouseLeave={() => setHovered(null)}
              onFocus={() => setHovered({ x, y })}
              onBlur={() => setHovered(null)}
              aria-label={describe(x, y, owner)}
              data-cy={`draft-cell-${x}-${y}`}
              data-piece={owner}
            />
          )
        })}
      </div>
      <GridSvg
        layout={{ pieces: [...draft.pieces] }}
        className='pointer-events-none absolute inset-0 bg-transparent'
        cy='draft-grid'
        label='Brouillon'
        piecePrefix='draft-piece'
        preview={preview}
        onPieceClick={draft.remove}
      />
    </div>
  )
}
