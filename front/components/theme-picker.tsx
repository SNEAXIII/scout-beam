'use client'

import { Moon, Sun } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import { useTheme } from '@/app/lib/theme'

export default function ThemePicker() {
  const { t } = useI18n()
  const { theme, toggle } = useTheme()

  return (
    <Button
      variant='ghost'
      size='sm'
      onClick={toggle}
      className='px-2'
      aria-label={t.theme.switch}
      title={t.theme.switch}
      data-cy='theme-picker'
      data-theme={theme}
    >
      {theme === 'dark' ? <Sun className='size-4' /> : <Moon className='size-4' />}
    </Button>
  )
}
