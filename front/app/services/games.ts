import { apiRequest } from '@/app/lib/api'
import type { BeamColour } from '@/app/lib/colours'
import type { Layout } from '@/app/lib/pieces'

export type GameStatus = 'in_progress' | 'solved' | 'surrendered'

export interface GameSummary {
  id: string
  status: GameStatus
  created_at: string
  solved_at: string | null
}

export const listGames = (): Promise<GameSummary[]> => apiRequest<GameSummary[]>('/games')

export interface Shot {
  entry: number
  exit_entry: number
  colour: BeamColour
}

export interface GameView extends GameSummary {
  /** In firing order. */
  shots: Shot[]
  /** Only once over: solved or surrendered. */
  layout?: Layout
}

export const createGame = (): Promise<GameView> =>
  apiRequest<GameView>('/games', { method: 'POST' })

export const getGame = (id: string): Promise<GameView> => apiRequest<GameView>(`/games/${id}`)

export interface PropositionVerdict {
  correct: boolean
  /** Only when correct. */
  layout?: Layout
}
