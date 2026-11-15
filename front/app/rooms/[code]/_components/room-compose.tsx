'use client'

import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import type { RoomView } from '@/app/services/rooms'
import KeyboardLayoutToggle from '@/app/games/[id]/_components/keyboard-layout-toggle'
import { useKeyboardLayout } from '@/app/games/[id]/_viewmodels/use-keyboard-layout'
import { useCompose } from '../_viewmodels/use-compose'
import ComposeBoard from './compose-board'
import RoomPlayers from './room-players'

interface RoomComposeProps {
  readonly room: RoomView
  readonly onRoom: (room: RoomView) => void
}

/** A Duel's composition, laid out like the game page it leads to. */
export default function RoomCompose({ room, onRoom }: RoomComposeProps) {
  const { t } = useI18n()
  const compose = useCompose(room.code, onRoom)
  const { keyboardLayout, toggle } = useKeyboardLayout()
  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-3 px-3 pb-3 md:px-4'>
      <header className='flex flex-wrap items-center gap-2 pt-2'>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/play'
            aria-label={t.rooms.back}
            title={t.rooms.back}
            data-cy='room-back'
          >
            <ArrowLeft />
          </Link>
        </Button>
        <h1
          className='text-xl font-semibold tracking-tight'
          data-cy='room-mode'
        >
          {t.rooms[room.mode]}
        </h1>
        <span
          className='font-mono text-sm tracking-widest text-muted-foreground'
          data-cy='room-code'
        >
          {room.code}
        </span>
        <div className='ml-auto flex items-center gap-2'>
          {!compose.hidden && (
            <KeyboardLayoutToggle
              keyboardLayout={keyboardLayout}
              onToggle={toggle}
            />
          )}
          <LanguageSwitcher />
        </div>
      </header>
      <div className='grid items-start gap-4 md:grid-cols-[minmax(0,1fr)_14rem]'>
        {compose.hidden ? (
          <p
            className='text-sm text-muted-foreground'
            data-cy='room-state-hidden'
          >
            {t.rooms.hidden}
          </p>
        ) : (
          <ComposeBoard
            code={room.code}
            keyboardLayout={keyboardLayout}
            pending={compose.pending}
            error={compose.error}
            onHide={(pieces) => void compose.hide(pieces)}
          />
        )}
        <aside className='flex flex-col gap-4 rounded-lg border border-border p-3'>
          <p className='text-sm text-muted-foreground'>{t.rooms.composing}</p>
          <RoomPlayers room={room} />
        </aside>
      </div>
    </main>
  )
}
