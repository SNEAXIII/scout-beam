'use client'

import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { ApiError } from '@/app/lib/api'
import { getAccessToken } from '@/app/lib/session'
import { fireShot, getGame, type GameView } from '@/app/services/games'
import GameBoard from './_components/game-board'
import GameHeader from './_components/game-header'
import ShotList from './_components/shot-list'
import { shotPaths } from './_components/shot-paths'

const message = (caught: unknown, fallback: string) =>
  caught instanceof Error ? caught.message : fallback

export default function GamePage() {
  const { id } = useParams<{ id: string }>()
  const router = useRouter()
  const [game, setGame] = useState<GameView | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const inFlight = useRef(new Set<number>())

  const load = useCallback(async () => {
    try {
      setGame(await getGame(id))
    } catch (caught) {
      setError(message(caught, 'Chargement impossible'))
    }
  }, [id])

  useEffect(() => {
    if (!getAccessToken()) {
      router.replace('/login')
      return
    }
    void load()
  }, [router, load])

  const fire = useCallback(
    async (entry: number) => {
      if (inFlight.current.has(entry)) return
      inFlight.current.add(entry)
      setError(null)
      try {
        const shot = await fireShot(id, entry)
        setGame((current) =>
          current && !current.shots.some((known) => known.entry === shot.entry)
            ? { ...current, shots: [...current.shots, shot] }
            : current
        )
      } catch (caught) {
        // 409: solved meanwhile, in another tab — reload to show the layout.
        if (caught instanceof ApiError && caught.status === 409) await load()
        else setError(message(caught, 'Tir impossible'))
      } finally {
        inFlight.current.delete(entry)
      }
    },
    [id, load]
  )

  const shots = useMemo(() => game?.shots ?? [], [game])
  const paths = useMemo(() => shotPaths(shots), [shots])
  const solved = game?.status === 'solved'

  return (
    <main className='mx-auto flex max-w-5xl flex-col gap-6 p-4 md:p-8'>
      <GameHeader status={game?.status ?? null} />
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='game-error'
        >
          {error}
        </p>
      )}
      {game === null && !error && <p className='text-sm text-muted-foreground'>Chargement…</p>}
      {game && (
        <div className='grid gap-6 md:grid-cols-[minmax(0,1fr)_16rem]'>
          <div className='flex flex-col gap-3'>
            <GameBoard
              paths={paths}
              layout={game.layout}
              disabled={solved}
              highlighted={highlighted}
              onFire={(entry) => void fire(entry)}
              onHighlight={setHighlighted}
            />
            {solved && (
              <p
                className='text-sm text-muted-foreground'
                data-cy='game-solved'
              >
                Partie résolue : voici les pièces cachées. Le tir est désactivé.
              </p>
            )}
          </div>
          <ShotList
            shots={shots}
            paths={paths}
            highlighted={highlighted}
            onHighlight={setHighlighted}
          />
        </div>
      )}
    </main>
  )
}
