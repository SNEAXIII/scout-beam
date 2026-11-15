'use client'

import { EyeOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import { GRID_COLUMNS, GRID_ROWS, SHAPES, type Piece } from '@/app/lib/pieces'
import type { RoomView } from '@/app/services/rooms'
import DraftBoard from '@/app/games/[id]/_components/draft-board'
import DraftToolbar from '@/app/games/[id]/_components/draft-toolbar'
import HotkeyLegend from '@/app/games/[id]/_components/hotkey-legend'
import KeyboardLayoutToggle from '@/app/games/[id]/_components/keyboard-layout-toggle'
import { useDraft } from '@/app/games/[id]/_viewmodels/use-draft'
import { useKeyboardLayout } from '@/app/games/[id]/_viewmodels/use-keyboard-layout'
import { composeKey, useCompose } from '../_viewmodels/use-compose'

interface RoomComposeProps {
  readonly code: string
  readonly onRoom: (room: RoomView) => void
}

/** A Duel's composition: the five pieces the opponent will hunt, then the wait for theirs. */
export default function RoomCompose({ code, onRoom }: RoomComposeProps) {
  const { t } = useI18n()
  const compose = useCompose(code, onRoom)
  if (compose.hidden)
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy='room-state-hidden'
      >
        {t.rooms.hidden}
      </p>
    )
  return (
    <ComposeBoard
      code={code}
      pending={compose.pending}
      error={compose.error}
      onHide={(pieces) => void compose.hide(pieces)}
    />
  )
}

interface ComposeBoardProps {
  readonly code: string
  readonly pending: boolean
  readonly error: string | null
  readonly onHide: (pieces: readonly Piece[]) => void
}

function ComposeBoard({ code, pending, error, onHide }: ComposeBoardProps) {
  const { t } = useI18n()
  const { keyboardLayout, toggle } = useKeyboardLayout()
  const draft = useDraft(composeKey(code), keyboardLayout)
  const complete = draft.pieces.length === SHAPES.length
  return (
    <div
      className='flex flex-col gap-3'
      data-cy='room-compose'
    >
      <p className='text-sm text-muted-foreground'>{t.rooms.composing}</p>
      <div className='flex flex-wrap items-start justify-between gap-2'>
        <DraftToolbar
          draft={draft}
          keyboardLayout={keyboardLayout}
        />
        <KeyboardLayoutToggle
          keyboardLayout={keyboardLayout}
          onToggle={toggle}
        />
      </div>
      <div style={{ aspectRatio: `${GRID_COLUMNS} / ${GRID_ROWS}` }}>
        <DraftBoard draft={draft} />
      </div>
      <div className='flex flex-wrap items-center gap-2'>
        <Button
          size='sm'
          onClick={() => onHide(draft.pieces)}
          disabled={!complete || pending}
          data-cy='room-hide'
        >
          <EyeOff />
          {t.rooms.hide}
        </Button>
        <span
          className='text-sm text-muted-foreground'
          data-cy='draft-count'
        >
          {fill(t.draft.placed, { count: draft.pieces.length, total: SHAPES.length })}
        </span>
      </div>
      <HotkeyLegend keyboardLayout={keyboardLayout} />
      {error && (
        <p
          className='rounded-md bg-destructive px-3 py-2 text-sm text-destructive-foreground'
          role='alert'
          data-cy='room-hide-error'
        >
          {error}
        </p>
      )}
    </div>
  )
}
