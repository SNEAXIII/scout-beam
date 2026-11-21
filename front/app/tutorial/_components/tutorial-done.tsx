import Link from 'next/link'
import { BookOpen, CircleCheck, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import GameBoard, { BOARD_COLUMN } from '@/app/games/[id]/_components/game-board'
import TraceLines from '@/app/games/[id]/_components/trace-lines'
import TraceModeToggle from '@/app/games/[id]/_components/trace-mode-toggle'
import { shotPaths } from '@/app/games/[id]/_components/shot-paths'
import { useTraceMode } from '@/app/games/[id]/_viewmodels/use-trace-mode'
import { drawnTraces, useTraces } from '@/app/games/[id]/_viewmodels/use-traces'
import { ENTRIES } from '@/app/lib/entries'
import { EXAMPLE_LAYOUT } from '../_lib/script'
import { shotOf, type Tutorial } from '../_viewmodels/use-tutorial'

// Every entry fired, as a finished game shows them.
const EXAMPLE_PATHS = shotPaths(ENTRIES.map((entry) => shotOf(entry, EXAMPLE_LAYOUT)))

/** The end of the Tutorial: a finished five-piece game, its traces picked as in a real one. */
export default function TutorialDone({ tutorial }: { tutorial: Tutorial }) {
  const { t } = useI18n()
  const traces = useTraces(EXAMPLE_LAYOUT)
  const { mode, choose } = useTraceMode()
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
      <p className='text-sm text-muted-foreground'>{t.tutorial.example}</p>
      <GameBoard
        paths={EXAMPLE_PATHS}
        layout={EXAMPLE_LAYOUT}
        disabled
        highlighted={tutorial.highlighted}
        onFire={tutorial.fire}
        onHighlight={tutorial.setHighlighted}
        traceKeyAt={traces.keyAt}
        onSelect={traces.toggle}
      >
        <TraceLines
          traces={drawnTraces(traces, mode, tutorial.highlighted)}
          animated={mode === 'selected' ? traces.selected : null}
        />
      </GameBoard>
      <div className='flex flex-wrap items-center justify-between gap-2'>
        <TraceModeToggle
          mode={mode}
          onChange={choose}
        />
        <div className='flex flex-wrap gap-2'>
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
    </div>
  )
}
