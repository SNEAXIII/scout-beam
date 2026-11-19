import { fill, useI18n } from '@/app/i18n'
import { clock } from '@/app/lib/clock'
import type { BotGameResult } from '@/app/services/games'

const WIDTH = 320
const HEIGHT = 120
const PAD = 12

export default function ScoreCurve({ recent }: Readonly<{ recent: BotGameResult[] }>) {
  const { t } = useI18n()
  if (recent.length < 2)
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy='stats-curve-empty'
      >
        {t.profile.stats.curveEmpty}
      </p>
    )

  const max = Math.max(...recent.map((game) => game.score), 1)
  const points = recent.map((game, index) => ({
    game,
    x: PAD + (index * (WIDTH - 2 * PAD)) / (recent.length - 1),
    y: HEIGHT - PAD - (game.score / max) * (HEIGHT - 2 * PAD),
  }))

  return (
    <figure className='flex flex-col gap-1'>
      <svg
        viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
        className='w-full text-primary'
        role='img'
        aria-label={t.profile.stats.curveLabel}
        data-cy='stats-curve'
      >
        <polyline
          points={points.map(({ x, y }) => `${x},${y}`).join(' ')}
          fill='none'
          stroke='currentColor'
          strokeWidth={2}
          strokeLinejoin='round'
          opacity={0.6}
        />
        {points.map(({ game, x, y }, index) => (
          <circle
            key={index}
            cx={x}
            cy={y}
            r={4}
            className={game.surrendered ? 'fill-destructive' : 'fill-current'}
            data-cy='stats-curve-dot'
          >
            <title>
              {fill(game.surrendered ? t.profile.stats.dotSurrendered : t.profile.stats.dot, {
                score: clock(game.score),
                points: game.points,
              })}
            </title>
          </circle>
        ))}
      </svg>
      <figcaption
        aria-hidden
        className='text-xs text-muted-foreground'
      >
        {t.profile.stats.curveLabel}
      </figcaption>
    </figure>
  )
}
