'use client'

import { Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useI18n } from '@/app/i18n'
import type { useDaily } from '../_viewmodels/use-daily'

interface DailyPlayCardProps {
  readonly daily: ReturnType<typeof useDaily>
}

/** `YYYY-MM-DD` read as a calendar day, whatever the browser's time zone. */
const formatDay = (date: string, locale: string) =>
  new Intl.DateTimeFormat(locale, { dateStyle: 'full', timeZone: 'UTC' }).format(new Date(date))

export default function DailyPlayCard({ daily }: DailyPlayCardProps) {
  const { t, locale } = useI18n()
  const { today, date, setDate, starting, error, play } = daily
  return (
    <Card>
      <CardHeader>
        <CardTitle
          className='first-letter:uppercase'
          data-cy='daily-date'
        >
          {date ? formatDay(date, locale) : t.common.loading}
        </CardTitle>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <div className='flex flex-col gap-2'>
          <Label htmlFor='daily-day'>{t.daily.pickDay}</Label>
          <Input
            id='daily-day'
            type='date'
            max={today ?? undefined}
            value={date}
            onChange={(event) => setDate(event.target.value)}
            disabled={today === null}
            className='w-fit'
            data-cy='daily-day'
          />
        </div>
        <Button
          onClick={() => void play()}
          disabled={!date || starting}
          className='w-fit'
          data-cy='daily-play'
        >
          <Play />
          {t.daily.play}
        </Button>
        {error && (
          <p
            className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
            role='alert'
            data-cy='daily-error'
          >
            {error}
          </p>
        )}
      </CardContent>
    </Card>
  )
}
