import { Fragment } from 'react'
import { Hand } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import { KEYMAPS, type KeyboardLayout } from '@/app/lib/keyboard'

interface HotkeyLegendProps {
  readonly keyboardLayout: KeyboardLayout
}

const Key = ({ children }: { readonly children: React.ReactNode }) => (
  <kbd className='rounded border border-border px-1 font-sans text-[10px] leading-4 text-foreground'>
    {children}
  </kbd>
)

/** The draft's shortcuts in the chosen keyboard layout; on a touch screen, only the long press. */
export default function HotkeyLegend({ keyboardLayout }: HotkeyLegendProps) {
  const { t } = useI18n()
  const keys = KEYMAPS[keyboardLayout]
  const entries = [
    { keys: [keys.shapePrevious, keys.shapeNext], label: t.hotkeys.shape },
    { keys: [keys.colourPrevious, keys.colourNext], label: t.hotkeys.colour },
    { keys: ['r'], label: t.hotkeys.rotate },
    { keys: [keys.wheel], label: t.hotkeys.wheel },
    { keys: [t.hotkeys.rightClick, t.hotkeys.space], label: t.hotkeys.remove },
  ]
  return (
    <>
      <p
        className='flex items-center gap-2 rounded-md border border-border bg-muted px-3 py-2 text-sm font-medium text-foreground pointer-fine:hidden'
        data-cy='touch-legend'
      >
        <Hand className='size-4 shrink-0 text-primary' />
        <span>
          {t.hotkeys.holdToRemove}
          <br />
          {t.hotkeys.holdForWheel}
        </span>
      </p>
      <ul
        className='hidden flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground pointer-fine:flex'
        aria-label={t.hotkeys.legend}
        data-cy='hotkey-legend'
        data-keyboard-layout={keyboardLayout}
      >
        {entries.map((entry) => (
          <li
            key={entry.label}
            className='flex items-center gap-1'
          >
            {entry.keys.map((key, index) => (
              <Fragment key={key}>
                {index > 0 && <span aria-hidden>/</span>}
                <Key>{key.length === 1 ? key.toUpperCase() : key}</Key>
              </Fragment>
            ))}
            <span>{entry.label}</span>
          </li>
        ))}
      </ul>
    </>
  )
}
