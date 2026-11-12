import { ENTRIES, entryPlace } from '@/app/lib/entries'
import { GRID_COLUMNS, GRID_ROWS, type Layout } from '@/app/lib/pieces'
import DraftBoard from './draft-board'
import EntryButton from './entry-button'
import GridSvg from './grid-svg'
import type { ShotPaths } from './shot-paths'
import type { Draft } from '../_viewmodels/use-draft'

export interface GameBoardProps {
  readonly paths: ShotPaths
  readonly layout?: Layout
  /** Drawn on the grid while the game is in progress; the revealed layout replaces it. */
  readonly draft?: Draft
  readonly disabled: boolean
  readonly highlighted: string | null
  readonly onFire: (entry: number) => void
  readonly onHighlight: (key: string | null) => void
}

const RING_COLUMNS = GRID_COLUMNS + 2
const RING_ROWS = GRID_ROWS + 2
const GRID_AREA = { gridColumn: `2 / span ${GRID_COLUMNS}`, gridRow: `2 / span ${GRID_ROWS}` }

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
}: GameBoardProps) {
  return (
    <div
      className='grid w-full'
      style={{
        gridTemplateColumns: `repeat(${RING_COLUMNS}, minmax(0, 1fr))`,
        gridTemplateRows: `repeat(${RING_ROWS}, minmax(0, 1fr))`,
        aspectRatio: `${RING_COLUMNS} / ${RING_ROWS}`,
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
        />
      )}
      {ENTRIES.map((entry) => (
        <EntryButton
          key={entry}
          entry={entry}
          path={paths.atEntry(entry)}
          fired={paths.fired.has(entry)}
          highlighted={highlighted}
          disabled={disabled}
          onFire={onFire}
          onHighlight={onHighlight}
          style={ringCell(entry)}
        />
      ))}
    </div>
  )
}
