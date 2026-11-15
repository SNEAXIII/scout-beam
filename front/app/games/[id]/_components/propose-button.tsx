import { Send } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import type { Pace } from '../_viewmodels/use-pace'

interface ProposeButtonProps {
  readonly disabled: boolean
  /** A room game's: the button fills through the cooldown, disabled meanwhile. */
  readonly pace: Pace | null
  readonly onPropose: () => void
}

export default function ProposeButton({ disabled, pace, onPropose }: ProposeButtonProps) {
  const { t } = useI18n()
  const cooling = pace !== null && pace.propositionInMs > 0
  return (
    <Button
      size='sm'
      className='relative isolate overflow-hidden'
      onClick={onPropose}
      disabled={disabled || cooling}
      title={
        cooling
          ? fill(t.rooms.propositionIn, { seconds: Math.ceil(pace.propositionInMs / 1000) })
          : undefined
      }
      data-cy='propose'
      data-cooling={cooling}
    >
      {cooling && (
        <span
          aria-hidden
          className='absolute inset-0 -z-10 origin-left bg-primary-foreground/30 transition-transform duration-100 ease-linear'
          style={{ transform: `scaleX(${pace.cooldownProgress})` }}
        />
      )}
      <Send />
      {t.draft.propose}
    </Button>
  )
}
