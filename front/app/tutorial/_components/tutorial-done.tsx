import Link from 'next/link'
import { BookOpen, CircleCheck, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from '../_lib/script'
import { pastTraces, type Tutorial } from '../_viewmodels/use-tutorial'

const ALL_TRACES = pastTraces(TUTORIAL_STEPS.length)

/** The end of the Tutorial, as a solved game: the revealed pieces under every shot's Trace. */
export default function TutorialDone({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
      data-cy='tutorial-done'
    >
      <div className='flex items-start gap-3 rounded-md border border-border bg-card px-4 py-3'>
        <CircleCheck className='mt-0.5 size-6 shrink-0 text-primary' />
        <div className='flex flex-col gap-1'>
          <h2 className='font-semibold'>{t.tutorial.doneTitle}</h2>
          <p className='text-sm text-muted-foreground'>{t.tutorial.doneText}</p>
        </div>
      </div>
      <GameBoard
        paths={tutorial.paths}
        layout={TUTORIAL_LAYOUT}
        disabled
        highlighted={tutorial.highlighted}
        onFire={tutorial.fire}
        onHighlight={tutorial.setHighlighted}
      >
        <TraceLines
          traces={ALL_TRACES}
          misses
        />
      </GameBoard>
      <div className='flex flex-wrap justify-end gap-2'>
        <Button
          asChild
          variant='outline'
        >
          <Link
            href='/rules'
            data-cy='tutorial-rules'
          >
            <BookOpen />
            {t.tutorial.readRules}
          </Link>
        </Button>
        <Button asChild>
          <Link
            href='/'
            data-cy='tutorial-daily'
          >
            <Play />
            {t.tutorial.playDaily}
          </Link>
        </Button>
      </div>
    </div>
  )
}
