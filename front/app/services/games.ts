import { apiRequest } from '@/app/lib/api'
import type { BeamColour } from '@/app/lib/colours'
import type { Layout } from '@/app/lib/pieces'

export type GameStatus = 'in_progress' | 'solved' | 'surrendered' | 'lost'

export interface GameSummary {
  id: string
  status: GameStatus
  created_at: string
  solved_at: string | null
  mode: GameMode
  /** The other player's login, in a room only. */
  opponent: string | null
}

export type GameMode = 'bot' | 'daily' | 'race' | 'duel'

export const listGames = (): Promise<GameSummary[]> => apiRequest<GameSummary[]>('/games')

export interface Shot {
  entry: number
  exit_entry: number
  colour: BeamColour
}

export interface GameView extends Omit<GameSummary, 'mode' | 'opponent'> {
  /** The day it is the daily of, `YYYY-MM-DD`; null for a game against the bot. */
  daily_date: string | null
  /** The room it is played in; null outside a room. */
  room_code: string | null
  /** Each one adds a minute to a daily's score. */
  wrong_propositions: number
  /** In firing order. */
  shots: Shot[]
  /** Only once over. */
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
