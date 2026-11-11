import { fitsGrid, PIECE_COLOURS, SHAPES, type Piece, type PieceColour } from './pieces'

const key = (gameId: string) => `scout-beam.draft.${gameId}`

const isPiece = (value: unknown): value is Piece => {
  if (typeof value !== 'object' || value === null) return false
  const { shape, colour, x, y, rotation } = value as Record<string, unknown>
  return (
    SHAPES.includes(shape as Piece['shape']) &&
    PIECE_COLOURS.includes(colour as PieceColour) &&
    Number.isInteger(x) &&
    Number.isInteger(y) &&
    [0, 1, 2, 3].includes(rotation as number)
  )
}

/** The stored draft of a game, keeping only well-formed pieces inside the grid, one per shape. */
export function loadDraft(gameId: string): Piece[] {
  try {
    const stored: unknown = JSON.parse(localStorage.getItem(key(gameId)) ?? '[]')
    if (!Array.isArray(stored)) return []
    const pieces: Piece[] = []
    for (const piece of stored as unknown[])
      if (isPiece(piece) && fitsGrid(piece) && !pieces.some((kept) => kept.shape === piece.shape))
        pieces.push({ ...piece })
    return pieces
  } catch {
    return []
  }
}

/** Blocked storage only costs the draft on reload. */
export function saveDraft(gameId: string, pieces: readonly Piece[]): void {
  try {
    localStorage.setItem(key(gameId), JSON.stringify(pieces))
  } catch {
    // The draft lives on in the page.
  }
}

export function clearDraft(gameId: string): void {
  try {
    localStorage.removeItem(key(gameId))
  } catch {
    // Unreadable storage holds no draft either.
  }
}
