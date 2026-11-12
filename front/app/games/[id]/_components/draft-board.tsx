import { useMemo, useState } from 'react'
import { GRID_COLUMNS, GRID_ROWS, pieceCells, type Shape } from '@/app/lib/pieces'
import { fill, useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import GridSvg from './grid-svg'
import type { Draft } from '../_viewmodels/use-draft'

interface DraftBoardProps {
  readonly draft: Draft
  readonly style?: React.CSSProperties
}

const CELLS = Array.from({ length: GRID_ROWS }, (_, y) =>
  Array.from({ length: GRID_COLUMNS }, (_, x) => ({ x, y }))
).flat()

/** The draft grid: a click places the picked shape; a right click or Space removes a piece. */
export default function DraftBoard({ draft, style }: DraftBoardProps) {
  const { t } = useI18n()
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
      ? { piece: previewPiece, fits: draft.fits(previewPiece) }
      : undefined

  // Space removes on keydown so no click follows: it would place the piece just picked back up.
  const onCellKey = (event: React.KeyboardEvent, owner: Shape | undefined) => {
    if (event.key !== ' ' || (!owner && !event.repeat)) return
    event.preventDefault()
    if (owner) draft.remove(owner)
  }

  const describe = (x: number, y: number, owner: Shape | undefined) => {
    const cell = { x: x + 1, y: y + 1 }
    if (owner) return fill(t.draft.cellRemove, { ...cell, shape: t.shapes[owner] })
    if (!draft.shape) return fill(t.draft.cell, cell)
    return fill(t.draft.cellPlace, { ...cell, shape: t.shapes[draft.shape] })
  }

  return (
    <div
      className='relative size-full'
      style={style}
      onContextMenu={(event) => event.preventDefault()}
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
              onClick={owner ? undefined : () => draft.place(x, y)}
              onKeyDown={(event) => onCellKey(event, owner)}
              onContextMenu={owner ? () => draft.remove(owner) : undefined}
              onMouseEnter={() => setHovered({ x, y })}
              onMouseLeave={() => setHovered(null)}
              onFocus={() => setHovered({ x, y })}
              onBlur={() => setHovered(null)}
              aria-label={describe(x, y, owner)}
              aria-keyshortcuts={owner ? 'Space' : undefined}
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
        label={t.draft.title}
        piecePrefix='draft-piece'
        preview={preview}
        onPieceRemove={draft.remove}
      />
    </div>
  )
}
