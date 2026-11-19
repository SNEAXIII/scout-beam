import { ENTRIES, entryPlace } from '@/app/lib/entries'
import { GRID_COLUMNS, GRID_ROWS, type Layout } from '@/app/lib/pieces'
import TapHint from '@/components/tap-hint'
import DraftBoard from './draft-board'
import EntryButton from './entry-button'
import GridSvg from './grid-svg'
import type { ShotPaths } from './shot-paths'
import type { Draft } from '../_viewmodels/use-draft'

/** What a guided board points at: an entry to fire from, or a grid cell. */
export type BoardGuide = number | { readonly x: number; readonly y: number }

export interface GameBoardProps {
  readonly paths: ShotPaths
  readonly layout?: Layout
  /** Drawn on the grid while the game is in progress; the revealed layout replaces it. */
  readonly draft?: Draft
  readonly disabled: boolean
  readonly highlighted: string | null
  readonly onFire: (entry: number) => void
  readonly onHighlight: (key: string | null) => void
  /** Leaves the ring empty, its room kept: a Duel's layout is composed without entries. */
  readonly hideEntries?: boolean
  /** Once the game is over: the trace each entry lies on, picked instead of firing. */
  readonly traceKeyAt?: (entry: number) => string | undefined
  readonly onSelect?: (entry: number) => void
  /** Drawn over the revealed layout. */
  readonly children?: React.ReactNode
  /** Shows a tap hint there; an entry guide also disables every other entry. */
  readonly guide?: BoardGuide
}

const RING_COLUMNS = GRID_COLUMNS + 2
const RING_ROWS = GRID_ROWS + 2
/** The ring of entries is thinner than a cell, as a fraction of one. */
const RING = 0.6
const RATIO = (GRID_COLUMNS + 2 * RING) / (GRID_ROWS + 2 * RING)
const tracks = (cells: number) =>
  `minmax(0, ${RING}fr) repeat(${cells}, minmax(0, 1fr)) minmax(0, ${RING}fr)`
const GRID_AREA = { gridColumn: `2 / span ${GRID_COLUMNS}`, gridRow: `2 / span ${GRID_ROWS}` }

/** Caps the left column (toolbar, board, proposition) so it fits one screen height. */
export const BOARD_COLUMN: React.CSSProperties = {
  maxWidth: `max(20rem, calc((100dvh - 13.5rem) * ${RATIO}))`,
}

/** CSS grid lines (1-based) of an entry in the ring around the grid. */
function ringCell(entry: number): React.CSSProperties {
  const { edge, index } = entryPlace(entry)
  switch (edge) {
    case 'top':
      return { gridColumn: index + 2, gridRow: 1 }
    case 'bottom':
      return { gridColumn: index + 2, gridRow: RING_ROWS }
    case 'right':
      return { gridColumn: RING_COLUMNS, gridRow: index + 2 }
    case 'left':
      return { gridColumn: 1, gridRow: index + 2 }
  }
}

export default function GameBoard({
  paths,
  layout,
  draft,
  disabled,
  highlighted,
  onFire,
  onHighlight,
  hideEntries = false,
  traceKeyAt,
  onSelect,
  children,
  guide,
}: GameBoardProps) {
  const guidedEntry = typeof guide === 'number' ? guide : undefined
  return (
    <div
      className='relative mx-auto grid w-full'
      style={{
        gridTemplateColumns: tracks(GRID_COLUMNS),
        gridTemplateRows: tracks(GRID_ROWS),
        aspectRatio: RATIO,
      }}
      data-cy='game-board'
    >
      {draft ? (
        <DraftBoard
          draft={draft}
          style={GRID_AREA}
        />
      ) : (
        <GridSvg
          layout={layout}
          style={GRID_AREA}
          className='overflow-visible'
        >
          {layout && children}
        </GridSvg>
      )}
      {!hideEntries &&
        ENTRIES.map((entry) => (
          <EntryButton
            key={entry}
            entry={entry}
            path={paths.atEntry(entry)}
            traceKey={traceKeyAt?.(entry)}
            fired={paths.fired.has(entry)}
            highlighted={highlighted}
            disabled={disabled || (guidedEntry !== undefined && entry !== guidedEntry)}
            onFire={onFire}
            onSelect={onSelect}
            onHighlight={onHighlight}
            inline={['top', 'bottom'].includes(entryPlace(entry).edge)}
            style={ringCell(entry)}
          />
        ))}
      {guide !== undefined && (
        <TapHint
          style={
            typeof guide === 'number'
              ? ringCell(guide)
              : { gridColumn: guide.x + 2, gridRow: guide.y + 2 }
          }
        />
      )}
    </div>
  )
}
