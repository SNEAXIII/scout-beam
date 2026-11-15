import Link from 'next/link'
import { ChevronRight } from 'lucide-react'
import { Card } from '@/components/ui/card'
import type { GameSummary } from '@/app/services/games'
import type { RoomView } from '@/app/services/rooms'
import { fill, useI18n } from '@/app/i18n'
import GameStatusBadge from './game-status-badge'

const DATE_STYLE = { dateStyle: 'long', timeStyle: 'short' } as const
const ROW =
  'flex items-center justify-between gap-3 px-4 py-3 transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none'

interface GameListProps {
  readonly games: GameSummary[] | null
  /** Rooms without a game yet, listed first: waiting for an opponent or composing a Duel. */
  readonly rooms: readonly RoomView[]
}

export default function GameList({ games, rooms }: GameListProps) {
  const { locale, t } = useI18n()
  if (games === null) {
    return <p className='text-sm text-muted-foreground'>{t.common.loading}</p>
  }
  if (games.length === 0 && rooms.length === 0) {
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy='games-empty'
      >
        {t.play.empty}
      </p>
    )
  }

  return (
    <Card>
      <ul
        className='divide-y'
        data-cy='games-list'
      >
        {rooms.map((room) => (
          <li key={room.code}>
            <Link
              href={`/rooms/${room.code}`}
              className={ROW}
              data-cy={`room-row-${room.code}`}
            >
              <span className='flex flex-col gap-0.5'>
                <span className='text-sm font-mono tracking-widest'>{room.code}</span>
                <span className='text-xs text-muted-foreground'>
                  {t.play.modes[room.mode]}
                  {room.opponent && ` · ${fill(t.play.versus, { login: room.opponent })}`}
                </span>
              </span>
              <span className='flex items-center gap-2 text-xs text-muted-foreground'>
                {room.status === 'waiting' ? t.play.roomWaiting : t.play.roomComposing}
                <ChevronRight className='size-4' />
              </span>
            </Link>
          </li>
        ))}
        {games.map((game) => (
          <li key={game.id}>
            <Link
              href={`/games/${game.id}`}
              className={ROW}
              data-cy={`game-row-${game.id}`}
            >
              <span className='flex flex-col gap-0.5'>
                <span className='text-sm'>
                  {new Date(game.created_at).toLocaleString(locale, DATE_STYLE)}
                </span>
                <span
                  className='text-xs text-muted-foreground'
                  data-cy='game-mode'
                >
                  {t.play.modes[game.mode]}
                  {game.opponent && ` · ${fill(t.play.versus, { login: game.opponent })}`}
                </span>
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
