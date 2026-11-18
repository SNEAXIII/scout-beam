import { Layers, MousePointerClick } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import type { TraceMode } from '../_viewmodels/use-trace-mode'

interface TraceModeToggleProps {
  readonly mode: TraceMode
  readonly onChange: (mode: TraceMode) => void
}

const MODES = [
  { mode: 'selected', Icon: MousePointerClick, label: 'traceSelected' },
  { mode: 'all', Icon: Layers, label: 'traceAll' },
] as const

export default function TraceModeToggle({ mode, onChange }: TraceModeToggleProps) {
  const { t } = useI18n()
  return (
    <div
      className='flex shrink-0 rounded-md border p-0.5'
      role='group'
      aria-label={t.game.traceModes}
    >
      {MODES.map(({ mode: option, Icon, label }) => (
        <Button
          key={option}
          variant={mode === option ? 'secondary' : 'ghost'}
          size='sm'
          className='px-2.5'
          onClick={() => onChange(option)}
          aria-pressed={mode === option}
          data-cy={`trace-mode-${option}`}
        >
          <Icon />
          {t.game[label]}
        </Button>
      ))}
    </div>
  )
}
