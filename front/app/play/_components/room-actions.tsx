'use client'

import { useState, type FormEvent } from 'react'
import { useRouter } from 'next/navigation'
import { DoorOpen, Flag, Swords } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { errorText, useI18n } from '@/app/i18n'
import { createRoom, isRoomCode, ROOM_CODE_ALPHABET, type RoomMode } from '@/app/services/rooms'

const MODES = [
  { mode: 'race', Icon: Flag },
  { mode: 'duel', Icon: Swords },
] as const

/** Opens a Race or a Duel room, or an existing room by its code. */
export default function RoomActions() {
  const { t } = useI18n()
  const router = useRouter()
  const [code, setCode] = useState('')
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const open = async (mode: RoomMode) => {
    setPending(true)
    setError(null)
    try {
      const room = await createRoom(mode)
      router.push(`/rooms/${room.code}`)
    } catch (caught) {
      setError(errorText(t, caught, t.rooms.createFailed))
      setPending(false)
    }
  }

  const go = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (isRoomCode(code)) router.push(`/rooms/${code}`)
  }

  const invalid = [...code].some((char) => !ROOM_CODE_ALPHABET.includes(char))
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t.rooms.title}</CardTitle>
        <CardDescription>{t.rooms.description}</CardDescription>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <div className='flex flex-wrap gap-2'>
          {MODES.map(({ mode, Icon }) => (
            <Button
              key={mode}
              variant='secondary'
              onClick={() => void open(mode)}
              disabled={pending}
              data-cy={`new-room-${mode}`}
            >
              <Icon />
              {t.rooms[mode]}
            </Button>
          ))}
        </div>
        {error && (
          <p
            className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
            role='alert'
            data-cy='new-room-error'
          >
            {error}
          </p>
        )}
        <form
          className='flex flex-col gap-2'
          onSubmit={go}
          data-cy='room-code-form'
        >
          <Label htmlFor='room-code'>{t.rooms.code}</Label>
          <div className='flex gap-2'>
            <Input
              id='room-code'
              value={code}
              onChange={(event) => setCode(event.target.value.toUpperCase().trim())}
              maxLength={6}
              autoComplete='off'
              spellCheck={false}
              aria-invalid={invalid}
              aria-describedby={invalid ? 'room-code-invalid' : undefined}
              className='font-mono tracking-widest uppercase'
              data-cy='room-code-input'
            />
            <Button
              type='submit'
              variant='outline'
              disabled={!isRoomCode(code)}
              data-cy='room-code-open'
            >
              <DoorOpen />
              {t.rooms.open}
            </Button>
          </div>
          {invalid && (
            <p
              id='room-code-invalid'
              className='text-xs text-muted-foreground'
              data-cy='room-code-invalid'
            >
              {t.rooms.codeInvalid}
            </p>
          )}
        </form>
      </CardContent>
    </Card>
  )
}
