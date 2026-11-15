'use client'

import Link from 'next/link'
import { CircleHelp, Gamepad2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import DailyBoard from './_components/daily-board'
import DayRanking from './_components/day-ranking'
import { useDaily } from './_viewmodels/use-daily'

export default function HomePage() {
  const { t } = useI18n()
  const daily = useDaily()
  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex flex-wrap items-center gap-2 md:grid md:grid-cols-[1fr_auto_1fr]'>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.landing.title}</h1>
        <span className='text-lg font-medium text-muted-foreground'>{t.daily.title}</span>
        <div className='ml-auto flex items-center gap-2 md:justify-self-end'>
          <Input
            type='date'
            aria-label={t.daily.pickDay}
            title={t.daily.pickDay}
            max={daily.today ?? undefined}
            value={daily.date}
            onChange={(event) => daily.setDate(event.target.value)}
            disabled={daily.today === null}
            className='w-fit'
            data-cy='daily-day'
          />
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
      <div className='grid items-start gap-6 md:grid-cols-[minmax(0,1fr)_18rem]'>
        <DailyBoard daily={daily} />
        <DayRanking date={daily.date} />
      </div>
    </main>
  )
}
