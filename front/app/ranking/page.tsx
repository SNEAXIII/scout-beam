'use client'

import { useEffect, useEffectEvent, useState } from 'react'
import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { errorText, useI18n } from '@/app/i18n'
import { getRanking, type GlobalRankingEntry, type Ranking } from '@/app/services/daily'
import RankingTable from './_components/ranking-table'

export default function RankingPage() {
  const { t } = useI18n()
  const [ranking, setRanking] = useState<Ranking<GlobalRankingEntry> | null>(null)
  const [error, setError] = useState<string | null>(null)

  const failed = useEffectEvent((caught: unknown) =>
    setError(errorText(t, caught, t.common.loadFailed))
  )

  useEffect(() => {
    getRanking().then(setRanking, failed)
  }, [])

  return (
    <main className='mx-auto flex w-full max-w-2xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex flex-wrap items-center gap-2'>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/'
            aria-label={t.ranking.back}
            title={t.ranking.back}
            data-cy='ranking-back'
          >
            <ArrowLeft />
          </Link>
        </Button>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.ranking.title}</h1>
        <div className='ml-auto'>
          <LanguageSwitcher />
        </div>
      </header>
      {error ? (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='ranking-error'
        >
          {error}
        </p>
      ) : (
        <RankingTable
          ranking={ranking}
          headers={[t.ranking.points, t.ranking.dailiesSolved]}
          cells={(entry) => [entry.points, entry.dailies_solved]}
          cy='global-ranking'
        />
      )}
    </main>
  )
}
