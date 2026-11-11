import { useState } from 'react'
import { Send } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ApiError } from '@/app/lib/api'
import { SHAPES } from '@/app/lib/pieces'
import { submitProposition } from '@/app/services/games'
import DraftBoard from './draft-board'
import DraftToolbar from './draft-toolbar'
import { useDraft } from './use-draft'

interface DraftPanelProps {
  readonly gameId: string
  /** Called once the game is solved, by this proposition or meanwhile elsewhere; the page clears the draft. */
  readonly onSolved: () => void
}

/** A verdict holds for the draft it was given, and vanishes as soon as the draft changes. */
interface Verdict {
  readonly draft: string
  readonly error?: string
}

export default function DraftPanel({ gameId, onSolved }: DraftPanelProps) {
  const draft = useDraft(gameId)
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
        error: caught instanceof Error ? caught.message : 'Proposition impossible',
      })
    }
    setPending(false)
  }

  return (
    <Card data-cy='draft-panel'>
      <CardHeader>
        <CardTitle>Brouillon</CardTitle>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <DraftToolbar draft={draft} />
        <DraftBoard draft={draft} />
        <div className='flex flex-wrap items-center gap-3'>
          <Button
            onClick={() => void propose()}
            disabled={!complete || pending}
            data-cy='propose'
          >
            <Send />
            Proposer
          </Button>
          <span
            className='text-sm text-muted-foreground'
            data-cy='draft-count'
          >
            {draft.pieces.length} / {SHAPES.length} pièces placées
          </span>
        </div>
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
            Incorrect
          </p>
        )}
      </CardContent>
    </Card>
  )
}
