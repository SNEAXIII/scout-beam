import { Hourglass, Swords } from 'lucide-react'
import { fill, useI18n } from '@/app/i18n'
import type { RoomGame } from '../_viewmodels/use-room-game'

/** The opponent, their shot count once known, and while the opponent has priority the wait. */
export default function RoomLine({ room }: Readonly<{ room: RoomGame }>) {
  const { t } = useI18n()
  return (
    <div className='flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted-foreground'>
      {room.opponent && (
        <span
          className='flex items-center gap-1.5'
          data-cy='room-opponent'
        >
          <Swords className='size-4' />
          {fill(t.rooms.against, { login: room.opponent })}
        </span>
      )}
      {room.opponentShots !== null && (
        <span data-cy='opponent-shots'>
          {fill(t.rooms.opponentShots, { count: room.opponentShots })}
        </span>
      )}
      {room.waitMs > 0 && (
        <span
          className='flex items-center gap-1.5 font-medium text-foreground tabular-nums'
          role='status'
          data-cy='priority-countdown'
        >
          <Hourglass className='size-4' />
          {fill(t.rooms.priority, { seconds: Math.ceil(room.waitMs / 1000) })}
        </span>
      )}
    </div>
  )
}
