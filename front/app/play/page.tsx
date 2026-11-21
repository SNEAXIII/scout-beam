'use client'

import { useCallback, useEffect, useState } from 'react'
import { useI18n } from '@/app/i18n'
import { listGames, type GameSummary } from '@/app/services/games'
import { listRooms, type RoomView } from '@/app/services/rooms'
import GameList from './_components/game-list'
import PlayHeader from './_components/play-header'
import PlayModes from './_components/play-modes'

export default function PlayPage() {
  const { t } = useI18n()
  const [games, setGames] = useState<GameSummary[] | null>(null)
  const [rooms, setRooms] = useState<RoomView[]>([])
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    try {
      setError(null)
      // One after the other: two 401s at once would race for the refresh cookie.
      setGames(await listGames())
      setRooms(await listRooms())
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : t.common.loadFailed)
    }
  }, [t])

  // Signed out, the list answers 401: `apiRequest` tries the refresh cookie, else goes to /login.
  useEffect(() => {
    void load()
  }, [load])

  return (
    <main className='mx-auto flex w-full max-w-6xl flex-col gap-6 p-4 md:p-8'>
      <PlayHeader />
      <div className='grid items-start gap-6 lg:grid-cols-2'>
        <PlayModes />
        <section className='flex flex-col gap-3'>
          <h2 className='text-xl font-bold'>{t.play.myGames}</h2>
          {error && (
            <p
              className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              role='alert'
              data-cy='games-error'
            >
              {error}
            </p>
          )}
          {!error && (
            <GameList
              games={games}
              rooms={rooms}
            />
          )}
        </section>
      </div>
    </main>
  )
}
