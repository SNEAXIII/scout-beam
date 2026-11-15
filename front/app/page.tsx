'use client'

import Link from 'next/link'
import { CircleHelp, Gamepad2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import DailyPlayCard from './_components/daily-play-card'
import DayRanking from './_components/day-ranking'
import { useDaily } from './_viewmodels/use-daily'

export default function HomePage() {
  const { t } = useI18n()
  const daily = useDaily()
  return (
    <main className='mx-auto flex w-full max-w-2xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex flex-wrap items-center gap-2'>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.landing.title}</h1>
        <span className='text-muted-foreground'>{t.daily.title}</span>
        <div className='ml-auto flex items-center gap-2'>
          <Button
            asChild
            variant='outline'
          >
            <Link
              href='/play'
              data-cy='play-link'
            >
              <Gamepad2 />
              {t.play.title}
            </Link>
          </Button>
          <Button
            asChild
            variant='ghost'
            size='icon'
          >
            <Link
              href='/rules'
              aria-label={t.landing.rulesTitle}
              title={t.landing.rulesTitle}
              data-cy='rules-link'
            >
              <CircleHelp />
            </Link>
          </Button>
          <LanguageSwitcher />
        </div>
      </header>
      <DailyPlayCard daily={daily} />
      <DayRanking date={daily.date} />
    </main>
  )
}
