'use client'

import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import DailyPlayCard from './_components/daily-play-card'
import { useDaily } from './_viewmodels/use-daily'

export default function DailyPage() {
  const { t } = useI18n()
  const daily = useDaily()
  return (
    <main className='mx-auto flex w-full max-w-2xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex items-center gap-2'>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/'
            aria-label={t.daily.back}
            title={t.daily.back}
            data-cy='daily-back'
          >
            <ArrowLeft />
          </Link>
        </Button>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.daily.title}</h1>
        <div className='ml-auto'>
          <LanguageSwitcher />
        </div>
      </header>
      <DailyPlayCard daily={daily} />
    </main>
  )
}
