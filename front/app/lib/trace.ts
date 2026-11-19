/** Tracing a beam through a layout, ported from the engine (`api/src/engine/beam.rs`). */

import type { BeamColour } from './colours'
import {
  GRID_COLUMNS,
  GRID_ROWS,
  pieceCells,
  type CellCorner,
  type CellFill,
  type Layout,
  type PieceColour,
} from './pieces.ts'

type Direction = 'N' | 'E' | 'S' | 'W'

interface Cell {
  x: number
  y: number
}

/** A point in grid units: one unit per cell, origin at the grid's top-left. */
export interface TracePoint {
  x: number
  y: number
}

export interface Trace {
  entry: number
  exit: number
  colour: BeamColour
  points: TracePoint[]
}

const STEP_CAP = 4 * GRID_COLUMNS * GRID_ROWS

const DELTA: Record<Direction, Cell> = {
  N: { x: 0, y: -1 },
  E: { x: 1, y: 0 },
  S: { x: 0, y: 1 },
  W: { x: -1, y: 0 },
}

const REVERSE: Record<Direction, Direction> = { N: 'S', E: 'W', S: 'N', W: 'E' }

const STRAIGHT_SIDES: Record<CellCorner, readonly [Direction, Direction]> = {
  NE: ['N', 'E'],
  NW: ['N', 'W'],
  SE: ['S', 'E'],
  SW: ['S', 'W'],
}

const inGrid = ({ x, y }: Cell): boolean => x >= 0 && y >= 0 && x < GRID_COLUMNS && y < GRID_ROWS

function launch(entry: number): [Cell, Direction] {
  const right = GRID_COLUMNS + GRID_ROWS
  const bottom = right + GRID_COLUMNS
  if (entry <= GRID_COLUMNS) return [{ x: entry - 1, y: 0 }, 'S']
  if (entry <= right) return [{ x: GRID_COLUMNS - 1, y: entry - GRID_COLUMNS - 1 }, 'W']
  if (entry <= bottom) return [{ x: bottom - entry, y: GRID_ROWS - 1 }, 'N']
  return [{ x: 0, y: 2 * (GRID_COLUMNS + GRID_ROWS) - entry }, 'E']
}

function leaving({ x, y }: Cell, direction: Direction): number {
  const bottom = 2 * GRID_COLUMNS + GRID_ROWS
  if (direction === 'N') return x + 1
  if (direction === 'E') return GRID_COLUMNS + 1 + y
  if (direction === 'S') return bottom - x
  return 2 * (GRID_COLUMNS + GRID_ROWS) - y
}

function deflect(fill: CellFill, travel: Direction): Direction {
  if (fill === 'full') return REVERSE[travel]
  const [first, second] = STRAIGHT_SIDES[fill]
  const enteredThrough = REVERSE[travel]
  if (enteredThrough === first || enteredThrough === second) return REVERSE[travel]
  return enteredThrough === REVERSE[first] ? REVERSE[second] : REVERSE[first]
}

const MASKS: Record<PieceColour, number> = { red: 1, blue: 2, yellow: 4, white: 8 }
/** Indexed by the red, blue and yellow bits. */
const MIXES = ['white', 'red', 'blue', 'violet', 'yellow', 'orange', 'green', 'black'] as const

/** White only shows when nothing else was touched: it adds nothing to a mix. */
function mix(bounces: readonly PieceColour[]): BeamColour {
  const mask = bounces.reduce((acc, colour) => acc | MASKS[colour], 0)
  if (mask === 0) return 'none'
  return MIXES[mask & 7]
}

/** What covers each cell of the grid; cells outside it are dropped, the first piece wins a clash. */
function occupancy(layout: Layout): Map<string, [CellFill, PieceColour]> {
  const cells = new Map<string, [CellFill, PieceColour]>()
  for (const piece of layout.pieces) {
    for (const cell of pieceCells(piece)) {
      const key = `${cell.x},${cell.y}`
      if (inGrid(cell) && !cells.has(key)) cells.set(key, [cell.fill, piece.colour])
    }
  }
  return cells
}

/** `cell`'s centre moved half a cell towards `direction`, or the centre itself. */
function pointIn({ x, y }: Cell, direction?: Direction): TracePoint {
  const delta = direction ? DELTA[direction] : { x: 0, y: 0 }
  return { x: x + 0.5 + delta.x / 2, y: y + 0.5 + delta.y / 2 }
}

/** Appends `point`, dropping the last one when the polyline runs straight on through it. */
function push(points: TracePoint[], point: TracePoint): void {
  const last = points.at(-1)
  const before = points.at(-2)
  if (last && last.x === point.x && last.y === point.y) return
  if (last && before) {
    const [ax, ay] = [last.x - before.x, last.y - before.y]
    const [bx, by] = [point.x - last.x, point.y - last.y]
    if (ax * by === ay * bx && ax * bx + ay * by > 0) points.pop()
  }
  points.push(point)
}

/** Fires a beam from `entry`; `undefined` past the engine's step cap, which only a bug reaches. */
export function traceBeam(layout: Layout, entry: number): Trace | undefined {
  const cells = occupancy(layout)
  let [cell, direction] = launch(entry)
  const bounces: PieceColour[] = []
  const points: TracePoint[] = [pointIn(cell, REVERSE[direction])]

  for (let step = 0; step < STEP_CAP; step += 1) {
    const occupant = cells.get(`${cell.x},${cell.y}`)
    if (occupant) {
      const [fill, colour] = occupant
      bounces.push(colour)
      const turned = deflect(fill, direction)
      push(points, turned === REVERSE[direction] ? pointIn(cell, turned) : pointIn(cell))
      direction = turned
    }
    const next = { x: cell.x + DELTA[direction].x, y: cell.y + DELTA[direction].y }
    if (!inGrid(next)) {
      push(points, pointIn(cell, direction))
      return { entry, exit: leaving(cell, direction), colour: mix(bounces), points }
    }
    cell = next
  }
  return undefined
}
