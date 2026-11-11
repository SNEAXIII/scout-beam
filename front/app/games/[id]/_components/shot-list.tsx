import { ArrowRight } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import type { Shot } from '@/app/services/games'
import BeamMarker from './beam-marker'
import type { ShotPaths } from './shot-paths'

interface ShotListProps {
  readonly shots: readonly Shot[]
  readonly paths: ShotPaths
  readonly highlighted: string | null
  readonly onHighlight: (key: string | null) => void
}

export default function ShotList({ shots, paths, highlighted, onHighlight }: ShotListProps) {
  const { t } = useI18n()
  return (
    <Card className='flex min-h-0 flex-col'>
      <CardHeader>
        <CardTitle>{t.shots.title}</CardTitle>
      </CardHeader>
      <CardContent className='min-h-0 overflow-y-auto'>
        {shots.length === 0 ? (
          <p
            className='text-sm text-muted-foreground'
            data-cy='shots-empty'
          >
            {t.shots.empty}
          </p>
        ) : (
          <ul
            className='flex flex-col gap-0.5'
            data-cy='shot-list'
          >
            {shots.map((shot) => {
              const path = paths.ofShot(shot)
              return (
                <li key={shot.entry}>
                  {/* A button so keyboard and touch can light a shot's two ends, as hovering does. */}
                  <button
                    type='button'
                    className={cn(
                      'flex w-full items-center gap-3 rounded-md px-2 py-1 text-left text-sm transition-colors',
                      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
                      highlighted === path.key && 'bg-accent'
                    )}
                    onClick={() => onHighlight(path.key)}
                    onMouseEnter={() => onHighlight(path.key)}
                    onMouseLeave={() => onHighlight(null)}
                    onFocus={() => onHighlight(path.key)}
                    onBlur={() => onHighlight(null)}
                    data-cy={`shot-${shot.entry}`}
                    data-exit={shot.exit_entry}
                    data-colour={shot.colour}
                  >
                    <BeamMarker path={path} />
                    <span className='flex items-center gap-1.5 tabular-nums'>
                      {shot.entry}
                      <ArrowRight className='size-3.5 text-muted-foreground' />
                      {shot.exit_entry}
                    </span>
                    <span className='ml-auto text-muted-foreground'>{t.colours[shot.colour]}</span>
                  </button>
                </li>
              )
            })}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}
