'use client'

import { Languages } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import { Button } from '@/components/ui/button'

export default function LanguageSwitcher() {
  const { locale, setLocale, t } = useI18n()
  const next = locale === 'en' ? 'fr' : 'en'

  return (
    <Button
      variant='ghost'
      size='sm'
      onClick={() => setLocale(next)}
      aria-label={t.language.switch}
      title={t.language.switch}
      data-cy='language-switcher'
    >
      <Languages />
      {locale.toUpperCase()}
    </Button>
  )
}
