'use client'

import { useCallback, useEffect, useState } from 'react'
import { useRouter } from 'next/navigation'
import { getAccessToken } from '@/app/lib/session'
import { listGames, type GameSummary } from '@/app/services/games'
import GameList from './_components/game-list'
import HomeHeader from './_components/home-header'

export default function HomePage() {
  const router = useRouter()
  const [games, setGames] = useState<GameSummary[] | null>(null)
  const [error, setError] = useState<string | null>(null)

  const load = useCallback(async () => {
    try {
      setError(null)
      setGames(await listGames())
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Chargement impossible')
    }
  }, [])

  useEffect(() => {
    if (!getAccessToken()) {
      router.replace('/login')
      return
    }
    void load()
  }, [router, load])

  return (
    <main className='mx-auto flex max-w-2xl flex-col gap-6 p-4 md:p-8'>
      <HomeHeader />
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='games-error'
        >
          {error}
        </p>
      )}
      {!error && <GameList games={games} />}
    </main>
  )
}
