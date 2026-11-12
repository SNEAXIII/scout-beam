import Link from 'next/link'
import { ChevronRight } from 'lucide-react'
import { Card } from '@/components/ui/card'
import type { GameSummary } from '@/app/services/games'
import { useI18n } from '@/app/i18n'
import GameStatusBadge from './game-status-badge'

const DATE_STYLE = { dateStyle: 'long', timeStyle: 'short' } as const

interface GameListProps {
  readonly games: GameSummary[] | null
}

export default function GameList({ games }: GameListProps) {
  const { locale, t } = useI18n()
  if (games === null) {
    return <p className='text-sm text-muted-foreground'>{t.common.loading}</p>
  }
  if (games.length === 0) {
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy='games-empty'
      >
        {t.home.empty}
      </p>
    )
  }

  return (
    <Card>
      <ul
        className='divide-y'
        data-cy='games-list'
      >
        {games.map((game) => (
          <li key={game.id}>
            <Link
              href={`/games/${game.id}`}
              className='flex items-center justify-between gap-3 px-4 py-3 transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none'
              data-cy={`game-row-${game.id}`}
            >
              <span className='text-sm'>
                {new Date(game.created_at).toLocaleString(locale, DATE_STYLE)}
              </span>
              <span className='flex items-center gap-2'>
                <GameStatusBadge status={game.status} />
                <ChevronRight className='size-4 text-muted-foreground' />
              </span>
            </Link>
          </li>
        ))}
      </ul>
    </Card>
  )
}
