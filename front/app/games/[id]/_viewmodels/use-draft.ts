import { useCallback, useEffect, useState } from 'react'
import { loadDraft, saveDraft } from '@/app/lib/draft'
import { KEYMAPS, type KeyboardLayout } from '@/app/lib/keyboard'
import {
  fitsGrid,
  overlaps,
  PIECE_COLOURS,
  SHAPES,
  type Piece,
  type PieceColour,
  type Rotation,
  type Shape,
} from '@/app/lib/pieces'

const nextFree = (pieces: readonly Piece[]): Shape | null =>
  SHAPES.find((shape) => !pieces.some((piece) => piece.shape === shape)) ?? null

const wrap = <T>(items: readonly T[], index: number): T =>
  items[((index % items.length) + items.length) % items.length]

/** The next free shape from `from` in `step` direction, wrapping; `from` itself if none is free. */
const walkShape = (from: Shape, step: 1 | -1, pieces: readonly Piece[]): Shape => {
  const start = SHAPES.indexOf(from)
  for (let offset = 1; offset < SHAPES.length; offset++) {
    const candidate = wrap(SHAPES, start + step * offset)
    if (!pieces.some((piece) => piece.shape === candidate)) return candidate
  }
  return from
}

const typing = (target: EventTarget | null) =>
  target instanceof HTMLElement &&
  (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName))

export interface Draft {
  readonly pieces: readonly Piece[]
  /** The shape the next click places; null once all five are placed. */
  readonly shape: Shape | null
  readonly rotation: Rotation
  readonly colour: PieceColour
  /** The piece a click at `(x, y)` would place, if a shape is picked. */
  readonly pieceAt: (x: number, y: number) => Piece | null
  /** Whether `piece` lies inside the grid, clear of the placed pieces. */
  readonly fits: (piece: Piece) => boolean
  readonly selectShape: (shape: Shape) => void
  readonly rotate: () => void
  readonly selectColour: (colour: PieceColour) => void
  readonly place: (x: number, y: number) => void
  readonly remove: (shape: Shape) => void
}

/** The player's draft of a game, kept in storage; `R` rotates, the keymap walks shapes and colours. */
export function useDraft(gameId: string, keyboardLayout: KeyboardLayout): Draft {
  // Mounted once the game has loaded, so never during prerender: storage is readable here.
  const [pieces, setPieces] = useState<readonly Piece[]>(() => loadDraft(gameId))
  const [shape, setShape] = useState<Shape | null>(() => nextFree(pieces))
  const [rotation, setRotation] = useState<Rotation>(0)
  const [colour, setColour] = useState<PieceColour>('red')

  const rotate = useCallback(() => setRotation((turns) => ((turns + 1) % 4) as Rotation), [])

  useEffect(() => {
    const keys = KEYMAPS[keyboardLayout]
    const onKey = (event: KeyboardEvent) => {
      if (event.ctrlKey || event.metaKey || event.altKey || typing(event.target)) return
      const key = event.key.toLowerCase()
      if (key === keys.colourPrevious || key === keys.colourNext) {
        const step = key === keys.colourNext ? 1 : -1
        setColour((current) => wrap(PIECE_COLOURS, PIECE_COLOURS.indexOf(current) + step))
        return
      }
      if (shape === null) return
      if (key === 'r') rotate()
      else if (key === keys.shapePrevious || key === keys.shapeNext)
        setShape(walkShape(shape, key === keys.shapeNext ? 1 : -1, pieces))
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [keyboardLayout, pieces, rotate, shape])

  const update = (next: readonly Piece[]) => {
    setPieces(next)
    saveDraft(gameId, next)
  }

  const pieceAt = (x: number, y: number): Piece | null => shape && { shape, colour, x, y, rotation }

  const fits = (piece: Piece) => fitsGrid(piece) && !overlaps(piece, pieces)

  const place = (x: number, y: number) => {
    const piece = pieceAt(x, y)
    if (!piece || !fits(piece) || pieces.some((placed) => placed.shape === piece.shape)) return
    const next = [...pieces, piece]
    update(next)
    setShape(nextFree(next))
  }

  // Picking the removed piece back up lets the player move it one click later.
  const remove = (removed: Shape) => {
    const piece = pieces.find((placed) => placed.shape === removed)
    if (!piece) return
    update(pieces.filter((placed) => placed !== piece))
    setShape(piece.shape)
    setRotation(piece.rotation)
    setColour(piece.colour)
  }

  const selectShape = (picked: Shape) => {
    if (!pieces.some((placed) => placed.shape === picked)) setShape(picked)
  }

  return {
    pieces,
    shape,
    rotation,
    colour,
    pieceAt,
    fits,
    selectShape,
    rotate,
    selectColour: setColour,
    place,
    remove,
  }
}
