import { Keyboard } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import type { KeyboardLayout } from '@/app/lib/keyboard'

interface KeyboardLayoutToggleProps {
  readonly keyboardLayout: KeyboardLayout
  readonly onToggle: () => void
}

export default function KeyboardLayoutToggle({
  keyboardLayout,
  onToggle,
}: KeyboardLayoutToggleProps) {
  const { t } = useI18n()
  return (
    <Button
      variant='ghost'
      size='sm'
      onClick={onToggle}
      aria-label={t.keyboard.switch}
      title={t.keyboard.switch}
      data-cy='keyboard-layout'
      data-keyboard-layout={keyboardLayout}
    >
      <Keyboard />
      {t.keyboard[keyboardLayout]}
    </Button>
  )
}
