'use client'

import { useI18n, type Locale } from '@/app/i18n'
import { Button } from '@/components/ui/button'

const FLAGS: Record<Locale, string> = { en: '🇬🇧', fr: '🇫🇷' }

/** Shows the flag of the language it switches to. */
export default function LanguageSwitcher() {
  const { locale, setLocale, t } = useI18n()
  const next: Locale = locale === 'en' ? 'fr' : 'en'

  return (
    <Button
      variant='ghost'
      size='sm'
      onClick={() => setLocale(next)}
      className='px-2 text-lg'
      aria-label={t.language.switch}
      title={t.language.switch}
      data-cy='language-switcher'
    >
      {FLAGS[next]}
    </Button>
  )
}
