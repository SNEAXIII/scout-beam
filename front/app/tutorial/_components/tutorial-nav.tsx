import { ArrowLeft, ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import TapHint from '@/components/tap-hint'
import { useI18n } from '@/app/i18n'
import type { Tutorial } from '../_viewmodels/use-tutorial'

interface TutorialNavProps {
  readonly tutorial: Tutorial
  /** Replaces "Next", as "Start" on the intro or the Proposition on the last step. */
  readonly children?: React.ReactNode
  readonly nextDisabled?: boolean
  readonly hint?: boolean
}

/** "Previous" and "Next", together under the board. */
export default function TutorialNav({ tutorial, children, nextDisabled, hint }: TutorialNavProps) {
  const { t } = useI18n()
  return (
    <div className='flex flex-wrap items-center justify-end gap-2'>
      <Button
        size='sm'
        variant='outline'
        disabled={tutorial.step === 0}
        onClick={tutorial.previous}
        data-cy='tutorial-previous'
      >
        <ArrowLeft />
        {t.tutorial.previous}
      </Button>
      {children ?? (
        <span className='relative'>
          <Button
            size='sm'
            disabled={nextDisabled}
            onClick={tutorial.next}
            data-cy='tutorial-next'
          >
            {t.tutorial.next}
            <ArrowRight />
          </Button>
          {hint && <TapHint />}
        </span>
      )}
    </div>
  )
}
