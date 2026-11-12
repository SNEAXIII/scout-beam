import { useState } from 'react'
import { Send } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import type { KeyboardLayout } from '@/app/lib/keyboard'
import { SHAPES } from '@/app/lib/pieces'
import { submitProposition } from '@/app/services/games'
import DraftToolbar from './draft-toolbar'
import HotkeyLegend from './hotkey-legend'
import GameBoard, { BOARD_COLUMN, type GameBoardProps } from './game-board'
import { useDraft } from '../_viewmodels/use-draft'

interface DraftPanelProps extends Omit<GameBoardProps, 'layout' | 'draft' | 'disabled'> {
  readonly gameId: string
  readonly keyboardLayout: KeyboardLayout
  /** Called once the game is solved, by this proposition or meanwhile elsewhere; the page clears the draft. */
  readonly onSolved: () => void
}

/** A verdict holds for the draft it was given, and vanishes as soon as the draft changes. */
interface Verdict {
  readonly draft: string
  readonly error?: string
}

/** The game board with the draft on its grid, the tools to build it and the proposition. */
export default function DraftPanel({
  gameId,
  keyboardLayout,
  onSolved,
  ...board
}: DraftPanelProps) {
  const { t } = useI18n()
  const draft = useDraft(gameId, keyboardLayout)
  const [pending, setPending] = useState(false)
  const [verdict, setVerdict] = useState<Verdict | null>(null)
  const current = JSON.stringify(draft.pieces)
  const shown = verdict?.draft === current ? verdict : null
  const complete = draft.pieces.length === SHAPES.length

  // Stays pending once solved: the reload unmounts the panel.
  const propose = async () => {
    setPending(true)
    setVerdict(null)
    try {
      const { correct } = await submitProposition(gameId, draft.pieces)
      if (correct) return onSolved()
      setVerdict({ draft: current })
    } catch (caught) {
      if (caught instanceof ApiError && caught.status === 409) return onSolved()
      setVerdict({
        draft: current,
        error: caught instanceof Error ? caught.message : t.draft.proposeFailed,
      })
    }
    setPending(false)
  }

  return (
    <div
      className='mx-auto flex w-full flex-col gap-2'
      style={BOARD_COLUMN}
      data-cy='draft-panel'
    >
      <DraftToolbar
        draft={draft}
        keyboardLayout={keyboardLayout}
      />
      <GameBoard
        {...board}
        draft={draft}
        disabled={false}
      />
      <div className='flex flex-wrap items-center gap-2'>
        <Button
          size='sm'
          onClick={() => void propose()}
          disabled={!complete || pending}
          data-cy='propose'
        >
          <Send />
          {t.draft.propose}
        </Button>
        <span
          className='text-sm text-muted-foreground'
          data-cy='draft-count'
        >
          {fill(t.draft.placed, { count: draft.pieces.length, total: SHAPES.length })}
        </span>
      </div>
      <HotkeyLegend keyboardLayout={keyboardLayout} />
      {shown?.error && (
        <p
          className='rounded-md bg-destructive px-3 py-2 text-sm text-destructive-foreground'
          role='alert'
          data-cy='proposition-error'
        >
          {shown.error}
        </p>
      )}
      {shown && !shown.error && (
        <p
          className='rounded-md border border-border px-3 py-2 text-sm font-medium'
          role='status'
          data-cy='proposition-incorrect'
        >
          {t.draft.incorrect}
        </p>
      )}
    </div>
  )
}
