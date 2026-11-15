import { Crown, User } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import type { RoomView } from '@/app/services/rooms'

/** The host, then the opponent once seated; a non-member never sees the opponent. */
export default function RoomPlayers({ room }: Readonly<{ room: RoomView }>) {
  const { t } = useI18n()
  return (
    <div className='flex flex-col gap-2'>
      <h2 className='text-sm font-medium text-muted-foreground'>{t.rooms.players}</h2>
      <ul
        className='flex flex-col gap-1'
        data-cy='room-players'
      >
        <li
          className='flex items-center gap-2'
          data-cy='room-host'
        >
          <Crown className='size-4 text-muted-foreground' />
          {room.host}
          <span className='text-xs text-muted-foreground'>({t.rooms.host})</span>
        </li>
        {room.opponent && (
          <li
            className='flex items-center gap-2'
            data-cy='room-opponent'
          >
            <User className='size-4 text-muted-foreground' />
            {room.opponent}
          </li>
        )}
      </ul>
    </div>
  )
}
