'use client'

import { useMemo } from 'react'
import { EyeOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import type { KeyboardLayout } from '@/app/lib/keyboard'
import { SHAPES, type Piece } from '@/app/lib/pieces'
import DraftToolbar from '@/app/games/[id]/_components/draft-toolbar'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import HotkeyLegend from '@/app/games/[id]/_components/hotkey-legend'
import { shotPaths } from '@/app/games/[id]/_components/shot-paths'
import { useDraft } from '@/app/games/[id]/_viewmodels/use-draft'
import { composeKey } from '../_viewmodels/use-compose'

interface ComposeBoardProps {
  readonly code: string
  readonly keyboardLayout: KeyboardLayout
  readonly pending: boolean
  readonly error: string | null
  readonly onHide: (pieces: readonly Piece[]) => void
}

const noop = () => {}

/** The game page's left column, entries idle: the five pieces the opponent will hunt. */
export default function ComposeBoard({
  code,
  keyboardLayout,
  pending,
  error,
  onHide,
}: ComposeBoardProps) {
  const { t } = useI18n()
  const draft = useDraft(composeKey(code), keyboardLayout)
  const paths = useMemo(() => shotPaths([]), [])
  const complete = draft.pieces.length === SHAPES.length
  return (
    <div
      className='mx-auto flex w-full flex-col gap-2'
      style={BOARD_COLUMN}
      data-cy='room-compose'
    >
      <DraftToolbar
        draft={draft}
        keyboardLayout={keyboardLayout}
      />
      <GameBoard
        paths={paths}
        draft={draft}
        disabled
        highlighted={null}
        onFire={noop}
        onHighlight={noop}
      />
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
