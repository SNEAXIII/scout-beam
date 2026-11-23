'use client'

import DailyBoard from './_components/daily-board'
import DayRanking from './_components/day-ranking'
import TutorialCard from './_components/tutorial-card'
import { BOARD_COLUMN } from './games/[id]/_components/game-board'
import { useDaily } from './_viewmodels/use-daily'

// Board, gap, ranking and padding: no wider, so the whole page sits centred on wide screens.
const PAGE_WIDTH = { maxWidth: `min(64rem, calc(${BOARD_COLUMN.maxWidth} + 23.5rem))` }

export default function HomePage() {
  const daily = useDaily()
  return (
    <main
      className='mx-auto flex w-full flex-col gap-6 p-4 md:my-auto md:p-8'
      style={PAGE_WIDTH}
    >
      <TutorialCard />
      <div className='grid items-start gap-6 md:grid-cols-[minmax(0,1fr)_18rem]'>
        <DailyBoard daily={daily} />
        <DayRanking date={daily.date} />
      </div>
    </main>
  )
}
