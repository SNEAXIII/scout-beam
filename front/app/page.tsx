'use client'

import Link from 'next/link'
import { CircleHelp, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import DailyBoard from './_components/daily-board'
import DayRanking from './_components/day-ranking'
import TutorialCard from './_components/tutorial-card'
import { useDaily } from './_viewmodels/use-daily'

export default function HomePage() {
  const { t } = useI18n()
  const daily = useDaily()
  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex items-center justify-end gap-2'>
        <Button asChild>
          <Link
            href='/play'
            data-cy='play-link'
          >
            <Plus />
            {t.landing.play}
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
      </header>
      <TutorialCard />
      <div className='grid items-start gap-6 md:grid-cols-[minmax(0,1fr)_18rem]'>
        <DailyBoard daily={daily} />
        <DayRanking date={daily.date} />
      </div>
    </main>
  )
}
