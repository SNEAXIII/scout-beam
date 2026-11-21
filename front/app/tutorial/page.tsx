'use client'

import { RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import TutorialDone from './_components/tutorial-done'
import TutorialDraft from './_components/tutorial-draft'
import TutorialIntro from './_components/tutorial-intro'
import TutorialShots from './_components/tutorial-shots'
import { TUTORIAL_STEPS } from './_lib/script'
import { useTutorial, type Tutorial } from './_viewmodels/use-tutorial'

/** A shot's answer stays shown while its step explains it, so no step repeats the exit or colour. */
function useStepText({ current, phase, shot }: Tutorial): (string | undefined)[] {
  const { t } = useI18n()
  if (current?.kind !== 'shot') return [current && t.tutorial.steps[current.text]]
  if (!shot) return [fill(t.tutorial.fire, { entry: current.entry })]
  const answer = fill(shot.colour === 'none' ? t.tutorial.answerMiss : t.tutorial.answer, {
    exit: shot.exit_entry,
    colour: t.colours[shot.colour].toLowerCase(),
  })
  return phase === 'explain' ? [answer, t.tutorial.steps[current.text]] : [answer]
}

export default function TutorialPage() {
  const { t } = useI18n()
  const tutorial = useTutorial()
  const { step, current } = tutorial
  const text = useStepText(tutorial)

  return (
    <main className='mx-auto flex w-full max-w-3xl flex-col gap-3 px-4 pb-4'>
      <div className='flex items-center gap-2'>
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
        <TutorialDone tutorial={tutorial} />
      ) : (
        <>
          <div className='flex min-h-40 flex-col gap-1 rounded-md border border-border bg-card px-3 py-2 text-sm sm:min-h-24'>
            <div
              aria-live='polite'
              data-cy='tutorial-text'
            >
              {text.map((line, i) => (
                <p
                  key={line}
                  className={i === 0 && text.length > 1 ? 'font-medium' : undefined}
                >
                  {line}
                </p>
              ))}
            </div>
          </div>
          {current?.kind !== 'intro' && (
            <p
              className='flex items-center gap-2 text-xs text-muted-foreground'
              data-cy='tutorial-demo-note'
            >
              <span className='size-2 shrink-0 rounded-full bg-foreground motion-safe:animate-[trace-blink_1.2s_ease-in-out_infinite]' />
              {t.tutorial.demoNote}
            </p>
          )}
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
