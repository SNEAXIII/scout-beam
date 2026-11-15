import { fill, useI18n } from '@/app/i18n'
import type { Pace } from '../_viewmodels/use-pace'

/** A room game's shots left, and a bar filling towards the next earned one. */
export default function ShotAllowance({ pace }: Readonly<{ pace: Pace }>) {
  const { t } = useI18n()
  const progress = Math.min(1, Math.max(0, pace.shotProgress))
  const percent = Math.round(progress * 100)
  const empty = pace.shotsLeft <= 0
  return (
    <div
      className='flex items-center gap-3 text-sm'
      data-cy='shot-allowance'
      data-left={pace.shotsLeft}
    >
      <div
        className='h-1.5 flex-1 overflow-hidden rounded-full bg-muted'
        role='progressbar'
        aria-label={t.rooms.nextShotProgress}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={percent}
      >
        {/* Eases between the 100 ms ticks; a new key on each shot earned or fired skips the rewind. */}
        <div
          key={pace.shotsLeft}
          className='h-full origin-left bg-primary transition-transform duration-100 ease-linear'
          style={{ transform: `scaleX(${progress})` }}
        />
      </div>
      <span
        className={empty ? 'font-medium tabular-nums' : 'text-muted-foreground tabular-nums'}
        role='status'
      >
        {empty
          ? fill(t.rooms.nextShotIn, { seconds: Math.ceil(pace.nextShotInMs / 1000) })
          : fill(t.rooms.shotsLeft, { count: pace.shotsLeft })}
      </span>
    </div>
  )
}
