'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import { ArrowLeft, UserPlus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { cn } from '@/app/lib/utils'
import RoomCode from './_components/room-code'
import RoomCompose from './_components/room-compose'
import RoomPlayers from './_components/room-players'
import RoomState from './_components/room-state'
import { useRoom } from './_viewmodels/use-room'

export default function RoomPage() {
  const { t } = useI18n()
  const code = useParams<{ code: string }>().code.toUpperCase()
  const { room, member, joining, error, join, update } = useRoom(code)
  // A guest is refused every room route: signing up converts it and comes back here.
  const guest = error instanceof ApiError && error.status === 403

  return (
    <main
      className={cn(
        'mx-auto flex w-full flex-col gap-6 p-4 md:p-8',
        room?.status === 'composing' ? 'max-w-xl' : 'max-w-md'
      )}
    >
      <Button
        asChild
        variant='ghost'
        className='self-start'
      >
        <Link
          href='/play'
          data-cy='room-back'
        >
          <ArrowLeft />
          {t.rooms.back}
        </Link>
      </Button>
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='room-error'
        >
          {error.message}
        </p>
      )}
      {guest && (
        <Button
          asChild
          className='self-start'
        >
          <Link
            href={`/register?next=${encodeURIComponent('/rooms/' + code)}`}
            data-cy='room-register'
          >
            <UserPlus />
            {t.rooms.register}
          </Link>
        </Button>
      )}
      {!room && !error && <p className='text-sm text-muted-foreground'>{t.common.loading}</p>}
      {room && (
        <Card data-cy='room'>
          <CardHeader>
            <CardTitle data-cy='room-mode'>{t.rooms[room.mode]}</CardTitle>
          </CardHeader>
          <CardContent className='flex flex-col gap-6'>
            <RoomCode code={room.code} />
            <RoomPlayers room={room} />
            {member && room.status === 'composing' ? (
              <RoomCompose
                code={room.code}
                onRoom={update}
              />
            ) : (
              <RoomState
                room={room}
                member={member}
                joining={joining}
                onJoin={() => void join()}
              />
            )}
          </CardContent>
        </Card>
      )}
    </main>
  )
}
