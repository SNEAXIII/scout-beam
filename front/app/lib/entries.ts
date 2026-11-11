import { GRID_COLUMNS, GRID_ROWS } from './pieces'

export type Edge = 'top' | 'right' | 'bottom' | 'left'

export const ENTRY_COUNT = 2 * (GRID_COLUMNS + GRID_ROWS)
export const ENTRIES: readonly number[] = Array.from({ length: ENTRY_COUNT }, (_, i) => i + 1)

/**
 * Where an entry sits, as the engine numbers them: top 1–10 left to right, right 11–18 top to
 * bottom, bottom 19–28 right to left, left 29–36 bottom to top. `index` is the column or row.
 */
export function entryPlace(entry: number): { edge: Edge; index: number } {
  const right = GRID_COLUMNS
  const bottom = right + GRID_ROWS
  const left = bottom + GRID_COLUMNS
  if (entry <= right) return { edge: 'top', index: entry - 1 }
  if (entry <= bottom) return { edge: 'right', index: entry - right - 1 }
  if (entry <= left) return { edge: 'bottom', index: left - entry }
  return { edge: 'left', index: ENTRY_COUNT - entry }
}
