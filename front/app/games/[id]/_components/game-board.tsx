import { ENTRIES, entryPlace } from '@/app/lib/entries'
import { GRID_COLUMNS, GRID_ROWS, type Layout } from '@/app/lib/pieces'
import EntryButton from './entry-button'
import GridSvg from './grid-svg'
import type { ShotPaths } from './shot-paths'

interface GameBoardProps {
  readonly paths: ShotPaths
  readonly layout?: Layout
  readonly disabled: boolean
  readonly highlighted: string | null
  readonly onFire: (entry: number) => void
  readonly onHighlight: (key: string | null) => void
}

const RING_COLUMNS = GRID_COLUMNS + 2
const RING_ROWS = GRID_ROWS + 2

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
      <GridSvg
        layout={layout}
        style={{ gridColumn: `2 / span ${GRID_COLUMNS}`, gridRow: `2 / span ${GRID_ROWS}` }}
      />
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
