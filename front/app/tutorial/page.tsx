'use client'

import Link from 'next/link'
import { ArrowLeft, RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import TutorialDone from './_components/tutorial-done'
import TutorialDraft from './_components/tutorial-draft'
import TutorialIntro from './_components/tutorial-intro'
import TutorialShots from './_components/tutorial-shots'
import { TUTORIAL_STEPS } from './_lib/script'
import { useTutorial, type Tutorial } from './_viewmodels/use-tutorial'

/** A shot step reads its guide, then its bare answer, then the step's explanation. */
function useStepText({ current, phase, shot }: Tutorial) {
  const { t } = useI18n()
  if (current?.kind !== 'shot' || phase === 'explain')
    return current && t.tutorial.steps[current.text]
  if (!shot) return fill(t.tutorial.fire, { entry: current.entry })
  return fill(shot.colour === 'none' ? t.tutorial.answerMiss : t.tutorial.answer, {
    exit: shot.exit_entry,
    colour: t.colours[shot.colour].toLowerCase(),
  })
}

export default function TutorialPage() {
  const { t } = useI18n()
  const tutorial = useTutorial()
  const { step, current } = tutorial
  const text = useStepText(tutorial)

  return (
    <main className='mx-auto flex w-full max-w-3xl flex-col gap-3 px-4 pb-4'>
      <div className='flex items-center gap-2'>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/'
            aria-label={t.tutorial.back}
            title={t.tutorial.back}
            data-cy='tutorial-back'
          >
            <ArrowLeft />
          </Link>
        </Button>
        <h1 className='text-lg font-semibold'>{t.tutorial.title}</h1>
        {!tutorial.done && (
          <span
            className='text-sm text-muted-foreground'
            data-cy='tutorial-step'
          >
            {fill(t.tutorial.step, { step: step + 1, total: TUTORIAL_STEPS.length })}
          </span>
        )}
        <Button
          variant='ghost'
          size='sm'
          className='ml-auto'
          onClick={tutorial.restart}
          data-cy='tutorial-restart'
        >
          <RotateCcw />
          {t.tutorial.restart}
        </Button>
      </div>
      {tutorial.done ? (
        <TutorialDone />
      ) : (
        <>
          <div className='flex flex-col gap-1 rounded-md border border-border bg-card px-3 py-2 text-sm'>
            <p
              aria-live='polite'
              data-cy='tutorial-text'
            >
              {text}
            </p>
          </div>
          {current?.kind === 'intro' && <TutorialIntro tutorial={tutorial} />}
          {current?.kind === 'shot' && <TutorialShots tutorial={tutorial} />}
          {(current?.kind === 'draft' || current?.kind === 'propose') && (
            <TutorialDraft tutorial={tutorial} />
          )}
        </>
      )}
    </main>
  )
}
