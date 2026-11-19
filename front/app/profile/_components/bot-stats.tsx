'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { clock } from '@/app/lib/clock'
import { useBotStats } from '../_viewmodels/use-bot-stats'
import ScoreCurve from './score-curve'

export default function BotStats() {
  const { t } = useI18n()
  const { stats, failed } = useBotStats()
  const labels = t.profile.stats
  const figures = stats && [
    { label: labels.played, value: stats.played, cy: 'stats-played' },
    { label: labels.solved, value: stats.solved, cy: 'stats-solved' },
    { label: labels.surrendered, value: stats.surrendered, cy: 'stats-surrendered' },
    {
      label: labels.best,
      value: stats.best_seconds === null ? '—' : clock(stats.best_seconds),
      cy: 'stats-best',
    },
    { label: labels.average, value: stats.average_points ?? '—', cy: 'stats-average' },
  ]

  return (
    <Card data-cy='bot-stats'>
      <CardHeader>
        <CardTitle>{labels.title}</CardTitle>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        {figures ? (
          <>
            <dl className='grid grid-cols-2 gap-3 sm:grid-cols-3'>
              {figures.map(({ label, value, cy }) => (
                <div key={cy}>
                  <dt className='text-xs text-muted-foreground'>{label}</dt>
                  <dd
                    className='text-lg font-semibold tabular-nums'
                    data-cy={cy}
                  >
                    {value}
                  </dd>
                </div>
              ))}
            </dl>
            <ScoreCurve recent={stats.recent} />
          </>
        ) : (
          <p className='text-sm text-muted-foreground'>
            {failed ? t.common.loadFailed : t.common.loading}
          </p>
        )}
      </CardContent>
    </Card>
  )
}
