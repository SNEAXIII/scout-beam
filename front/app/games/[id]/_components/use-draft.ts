import { useCallback, useEffect, useState } from 'react'
import { loadDraft, saveDraft } from '@/app/lib/draft'
import {
  fitsGrid,
  SHAPES,
  type Piece,
  type PieceColour,
  type Rotation,
  type Shape,
} from '@/app/lib/pieces'

const nextFree = (pieces: readonly Piece[]): Shape | null =>
  SHAPES.find((shape) => !pieces.some((piece) => piece.shape === shape)) ?? null

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
  readonly selectShape: (shape: Shape) => void
  readonly rotate: () => void
  readonly selectColour: (colour: PieceColour) => void
  readonly place: (x: number, y: number) => void
  readonly remove: (shape: Shape) => void
}

/** The player's draft of a game, kept in storage; `R` rotates the picked shape. */
export function useDraft(gameId: string): Draft {
  // Mounted once the game has loaded, so never during prerender: storage is readable here.
  const [pieces, setPieces] = useState<readonly Piece[]>(() => loadDraft(gameId))
  const [shape, setShape] = useState<Shape | null>(() => nextFree(pieces))
  const [rotation, setRotation] = useState<Rotation>(0)
  const [colour, setColour] = useState<PieceColour>('red')

  const rotate = useCallback(() => setRotation((turns) => ((turns + 1) % 4) as Rotation), [])

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() !== 'r' || event.ctrlKey || event.metaKey || event.altKey) return
      if (shape === null || typing(event.target)) return
      rotate()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [rotate, shape])

  const update = (next: readonly Piece[]) => {
    setPieces(next)
    saveDraft(gameId, next)
  }

  const pieceAt = (x: number, y: number): Piece | null => shape && { shape, colour, x, y, rotation }

  const place = (x: number, y: number) => {
    const piece = pieceAt(x, y)
    if (!piece || !fitsGrid(piece) || pieces.some((placed) => placed.shape === piece.shape)) return
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
    selectShape,
    rotate,
    selectColour: setColour,
    place,
    remove,
  }
}
