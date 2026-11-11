import { fill, useI18n, type Translations } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import BeamMarker from './beam-marker'
import type { ShotPath } from './shot-paths'

interface EntryButtonProps {
  readonly entry: number
  readonly path?: ShotPath
  readonly fired: boolean
  readonly highlighted: string | null
  readonly disabled: boolean
  readonly onFire: (entry: number) => void
  readonly onHighlight: (key: string | null) => void
  readonly style: React.CSSProperties
}

/** "Entry 3, beam A: 3 → 17, Red", read from this end. */
function describe(t: Translations, entry: number, path?: ShotPath): string {
  if (!path) return fill(t.game.entry, { entry })
  const other = path.from === entry ? path.to : path.from
  const colour = t.colours[path.colour]
  return fill(t.game.entryShot, { entry, label: path.label, other, colour })
}

export default function EntryButton({
  entry,
  path,
  fired,
  highlighted,
  disabled,
  onFire,
  onHighlight,
  style,
}: EntryButtonProps) {
  const { t } = useI18n()
  const lit = highlighted !== null && path?.key === highlighted
  const hover = () => onHighlight(path?.key ?? null)
  const leave = () => onHighlight(null)

  return (
    <button
      type='button'
      style={style}
      className={cn(
        'm-px flex flex-col items-center justify-center rounded-md text-[9px] leading-none text-muted-foreground transition-colors md:m-0.5 md:gap-0.5 md:text-xs',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50',
        !fired && !disabled && 'hover:bg-accent hover:text-accent-foreground',
        fired && 'text-foreground',
        lit && 'bg-accent ring-2 ring-ring',
        highlighted !== null && !lit && 'opacity-40',
        disabled && !fired && 'cursor-not-allowed'
      )}
      onClick={() => {
        if (!fired && !disabled) onFire(entry)
      }}
      onMouseEnter={hover}
      onMouseLeave={leave}
      onFocus={hover}
      onBlur={leave}
      aria-disabled={fired || disabled}
      aria-label={describe(t, entry, path)}
      data-cy={`entry-${entry}`}
      data-fired={fired}
      data-labels={path?.label ?? ''}
      data-colour={path?.colour}
    >
      <span className='tabular-nums'>{entry}</span>
      {path && (
        <BeamMarker
          path={path}
          className='size-3.5 text-[8px] md:size-5 md:text-[10px]'
        />
      )}
    </button>
  )
}
