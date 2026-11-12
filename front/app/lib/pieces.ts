/** Piece geometry, ported from the engine (`api/src/engine/piece.rs`): same offsets, same rotation. */

export const GRID_COLUMNS = 10
export const GRID_ROWS = 8

export type Shape = 'square' | 'bar' | 'spike' | 'corner' | 'diamond'
export type PieceColour = 'red' | 'blue' | 'yellow' | 'white'
/** Quarter turns clockwise. */
export type Rotation = 0 | 1 | 2 | 3
export type CellCorner = 'NE' | 'NW' | 'SE' | 'SW'
/** A full cell, or the corner a half cell fills. */
export type CellFill = 'full' | CellCorner

export interface Piece {
  shape: Shape
  colour: PieceColour
  /** Top-left of the rotated bounding box. */
  x: number
  y: number
  rotation: Rotation
}

export interface Layout {
  pieces: Piece[]
}

/** A covered cell, in grid coordinates that may lie outside the grid. */
export interface PieceCell {
  x: number
  y: number
  fill: CellFill
}

export const SHAPES: readonly Shape[] = ['square', 'bar', 'spike', 'corner', 'diamond']

export const PIECE_COLOURS: readonly PieceColour[] = ['red', 'blue', 'yellow', 'white']

/** The names the API uses in its messages. */
const BASE_CELLS: Record<Shape, readonly PieceCell[]> = {
  square: [{ x: 0, y: 0, fill: 'full' }],
  bar: [
    { x: 0, y: 0, fill: 'full' },
    { x: 1, y: 0, fill: 'full' },
  ],
  spike: [{ x: 0, y: 0, fill: 'SW' }],
  corner: [
    { x: 0, y: 0, fill: 'SW' },
    { x: 0, y: 1, fill: 'full' },
    { x: 1, y: 1, fill: 'SW' },
  ],
  diamond: [
    { x: 0, y: 0, fill: 'SE' },
    { x: 1, y: 0, fill: 'SW' },
    { x: 0, y: 1, fill: 'NE' },
    { x: 1, y: 1, fill: 'NW' },
  ],
}

const CLOCKWISE: Record<CellCorner, CellCorner> = { NW: 'NE', NE: 'SE', SE: 'SW', SW: 'NW' }

const rotateFill = (fill: CellFill): CellFill => (fill === 'full' ? fill : CLOCKWISE[fill])

/** The cells `piece` covers once rotated, its bounding box anchored at `(x, y)`. */
export function pieceCells(piece: Piece): PieceCell[] {
  let cells = BASE_CELLS[piece.shape].map((cell) => ({ ...cell }))
  for (let turn = 0; turn < piece.rotation; turn += 1) {
    cells = cells.map(({ x, y, fill }) => ({ x: -y, y: x, fill: rotateFill(fill) }))
  }
  const minX = Math.min(...cells.map((cell) => cell.x))
  const minY = Math.min(...cells.map((cell) => cell.y))
  return cells.map((cell) => ({
    x: cell.x + piece.x - minX,
    y: cell.y + piece.y - minY,
    fill: cell.fill,
  }))
}

/** Whether every cell of `piece` lies inside the grid. */
export const fitsGrid = (piece: Piece): boolean =>
  pieceCells(piece).every(({ x, y }) => x >= 0 && y >= 0 && x < GRID_COLUMNS && y < GRID_ROWS)

/** Whether `piece` covers a cell one of `others` covers, even a half cell's empty half. */
export function overlaps(piece: Piece, others: readonly Piece[]): boolean {
  const key = ({ x, y }: PieceCell) => `${x},${y}`
  const taken = new Set(others.flatMap((other) => pieceCells(other).map(key)))
  return pieceCells(piece).some((cell) => taken.has(key(cell)))
}

/** SVG `points` of a cell drawn with unit-sized cells. */
export function cellPoints({ x, y, fill }: PieceCell): string {
  const nw = [x, y]
  const ne = [x + 1, y]
  const se = [x + 1, y + 1]
  const sw = [x, y + 1]
  const corners: Record<CellFill, number[][]> = {
    full: [nw, ne, se, sw],
    NE: [nw, ne, se],
    NW: [sw, nw, ne],
    SE: [ne, se, sw],
    SW: [nw, sw, se],
  }
  return corners[fill].map(([px, py]) => `${px},${py}`).join(' ')
}
