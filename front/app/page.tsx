'use client'

import DailyBoard from './_components/daily-board'
import DayRanking from './_components/day-ranking'
import TutorialCard from './_components/tutorial-card'
import { useDaily } from './_viewmodels/use-daily'

export default function HomePage() {
  const daily = useDaily()
  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-6 p-4 md:p-8'>
      <TutorialCard />
      <div className='grid items-start gap-6 md:grid-cols-[minmax(0,1fr)_18rem]'>
        <DailyBoard daily={daily} />
        <DayRanking date={daily.date} />
      </div>
    </main>
  )
}
