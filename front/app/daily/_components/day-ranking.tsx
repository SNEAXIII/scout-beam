'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { Trophy } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import { clock } from '@/app/lib/clock'
import { getDayRanking, type DayRankingEntry, type Ranking } from '@/app/services/daily'
import RankingTable from '@/app/ranking/_components/ranking-table'

interface DayRankingProps {
  /** `YYYY-MM-DD`, or empty while no day is picked. */
  readonly date: string
}

interface Loaded {
  readonly date: string
  readonly ranking: Ranking<DayRankingEntry> | null
  readonly failed: boolean
}

/** The picked day's ranking: raw time, plus the penalty of its wrong propositions. */
export default function DayRanking({ date }: DayRankingProps) {
  const { t } = useI18n()
  const [loaded, setLoaded] = useState<Loaded | null>(null)

  useEffect(() => {
    if (!date) return
    let current = true
    getDayRanking(date).then(
      (ranking) => current && setLoaded({ date, ranking, failed: false }),
      () => current && setLoaded({ date, ranking: null, failed: true })
    )
    return () => {
      current = false
    }
  }, [date])

  const shown = loaded?.date === date ? loaded : null
  const time = (entry: DayRankingEntry) =>
    entry.wrong_propositions > 0
      ? `${clock(entry.seconds)} ${fill(t.daily.penalty, { count: entry.wrong_propositions })}`
      : clock(entry.seconds)

  return (
    <section className='flex flex-col gap-3'>
      <div className='flex flex-wrap items-center justify-between gap-2'>
        <h2 className='text-lg font-semibold tracking-tight'>{t.ranking.dayTitle}</h2>
        <Button
          asChild
          variant='ghost'
          size='sm'
        >
          <Link
            href='/ranking'
            data-cy='global-ranking-link'
          >
            <Trophy />
            {t.ranking.global}
          </Link>
        </Button>
      </div>
      {shown?.failed ? (
        <p
          className='text-sm text-muted-foreground'
          role='alert'
          data-cy='day-ranking-error'
        >
          {t.common.loadFailed}
        </p>
      ) : (
        <RankingTable
          ranking={shown?.ranking ?? null}
          headers={[t.ranking.time]}
          cells={(entry) => [time(entry)]}
          cy='day-ranking'
        />
      )}
    </section>
  )
}
