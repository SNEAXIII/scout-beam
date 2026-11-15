'use client'

import { LogIn } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import type { RoomView } from '@/app/services/rooms'

interface RoomStateProps {
  readonly room: RoomView
  readonly member: boolean
  readonly joining: boolean
  readonly onJoin: () => void
}

/** What the caller waits for, or the Join a free seat offers a non-member. */
export default function RoomState({ room, member, joining, onJoin }: RoomStateProps) {
  const { t } = useI18n()
  if (!member && room.status === 'waiting') {
    return (
      <Button
        onClick={onJoin}
        disabled={joining}
        data-cy='room-join'
      >
        <LogIn />
        {t.rooms.join}
      </Button>
    )
  }
  const lines = {
    waiting: t.rooms.waiting,
    composing: t.rooms.composing,
    playing: t.rooms.starting,
    finished: t.rooms.starting,
  }
  const line = member ? lines[room.status] : t.rooms.full
  return (
    <p
      className='text-sm text-muted-foreground'
      data-cy={`room-state-${member ? room.status : 'full'}`}
    >
      {line}
    </p>
  )
}
