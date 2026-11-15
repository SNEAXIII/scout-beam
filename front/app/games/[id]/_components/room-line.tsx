import { Swords } from 'lucide-react'
import { fill, useI18n } from '@/app/i18n'
import type { RoomGame } from '../_viewmodels/use-room-game'

/** The opponent, and their shot count once known. */
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
    </div>
  )
}
