import { ApiError, apiRequest, refreshSession } from '@/app/lib/api'
import { guestLogin } from './auth'
import type { GameView } from './games'

/** Today's date in Paris, as `YYYY-MM-DD`: the day changes on the API's clock, not the browser's. */
export const getToday = () => apiRequest<{ date: string }>('/daily/today').then(({ date }) => date)

/** The caller's game of that day, signing in as a guest first when there is no session at all. */
export async function playDaily(date: string): Promise<GameView> {
  const start = () =>
    apiRequest<GameView>(`/daily/${date}/game`, { method: 'POST', noRedirect: true })
  try {
    return await start()
  } catch (caught) {
    if (!(caught instanceof ApiError && caught.status === 401)) throw caught
  }
  if (!(await refreshSession())) await guestLogin()
  return start()
}

export interface RankingEntry {
  /** Equal scores or points share a rank. */
  rank: number
  login: string
}

export interface DayRankingEntry extends RankingEntry {
  /** Seconds plus 60 per wrong proposition; lower is better. */
  score: number
  seconds: number
  wrong_propositions: number
}

export interface GlobalRankingEntry extends RankingEntry {
  points: number
  dailies_solved: number
}

export interface Ranking<T extends RankingEntry> {
  top: T[]
  /** The caller's line, inside the top or not; null when signed out or unranked. */
  me: T | null
}

// Public routes: an expired access cookie only leaves `me` empty, so no refresh nor /login.
export const getDayRanking = (date: string) =>
  apiRequest<Ranking<DayRankingEntry>>(`/daily/${date}/ranking`, { noRedirect: true })

export const getRanking = () =>
  apiRequest<Ranking<GlobalRankingEntry>>('/ranking', { noRedirect: true })
