import { apiRequest } from '@/app/lib/api'

export type GameStatus = 'in_progress' | 'solved'

export interface GameSummary {
  id: string
  status: GameStatus
  created_at: string
  solved_at: string | null
}

export const listGames = (): Promise<GameSummary[]> => apiRequest<GameSummary[]>('/games')

/** The created game's view; only its summary fields are typed until the game page needs more. */
export const createGame = (): Promise<GameSummary> =>
  apiRequest<GameSummary>('/games', { method: 'POST' })
