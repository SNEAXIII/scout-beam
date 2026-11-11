import Link from 'next/link'
import { ChevronRight } from 'lucide-react'
import { Card } from '@/components/ui/card'
import type { GameSummary } from '@/app/services/games'
import GameStatusBadge from './game-status-badge'

const DATE_FORMAT = new Intl.DateTimeFormat('fr-FR', { dateStyle: 'long', timeStyle: 'short' })

interface GameListProps {
  readonly games: GameSummary[] | null
}

export default function GameList({ games }: GameListProps) {
  if (games === null) {
    return <p className='text-sm text-muted-foreground'>Chargement…</p>
  }
  if (games.length === 0) {
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy='games-empty'
      >
        Aucune partie pour l’instant. Lancez-en une !
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
              <span className='text-sm'>{DATE_FORMAT.format(new Date(game.created_at))}</span>
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
