import { useCallback, useEffect, useRef, useState } from 'react'
import { useRouter } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { clearDraft } from '@/app/lib/draft'
import { getAccessToken } from '@/app/lib/session'
import { fireShot, getGame, surrenderGame, type GameView } from '@/app/services/games'

const message = (caught: unknown, fallback: string) =>
  caught instanceof Error ? caught.message : fallback

/** A game, its shots and the player's two actions on it: firing and surrendering. */
export function useGame(id: string) {
  const { t } = useI18n()
  const router = useRouter()
  const [game, setGame] = useState<GameView | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [surrendering, setSurrendering] = useState(false)
  const inFlight = useRef(new Set<number>())

  const load = useCallback(async () => {
    try {
      setGame(await getGame(id))
    } catch (caught) {
      setError(message(caught, t.common.loadFailed))
    }
  }, [id, t])

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
        // 409: over meanwhile, in another tab — reload to show the layout.
        if (caught instanceof ApiError && caught.status === 409) await load()
        else setError(message(caught, t.game.shotFailed))
      } finally {
        inFlight.current.delete(entry)
      }
    },
    [id, load, t]
  )

  const surrender = useCallback(async () => {
    if (!window.confirm(t.game.surrenderConfirm)) return
    setSurrendering(true)
    setError(null)
    try {
      setGame(await surrenderGame(id))
    } catch (caught) {
      // 409: over meanwhile, in another tab — reload to show the layout.
      if (caught instanceof ApiError && caught.status === 409) await load()
      else setError(message(caught, t.game.surrenderFailed))
    } finally {
      setSurrendering(false)
    }
  }, [id, load, t])

  const over = game !== null && game.status !== 'in_progress'

  useEffect(() => {
    if (over) clearDraft(id)
  }, [over, id])

  return { game, error, over, surrendering, load, fire, surrender }
}
