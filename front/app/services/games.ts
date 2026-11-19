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
  /** In seconds; a finished Bot game only, like `points`. */
  score: number | null
  points: number | null
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

/** A room game only, by the server's clock: the shots left follow from these and the shot count. */
export interface PaceView {
  /** Since the game started. */
  elapsed_ms?: number
  /** Before the next proposition is taken; 0 when it is now. */
  proposition_in_ms?: number
}

export interface GameView extends Omit<GameSummary, 'mode' | 'opponent' | 'score' | 'points'>, PaceView {
  /** The day it is the daily of, `YYYY-MM-DD`; null for a game against the bot. */
  daily_date: string | null
  /** The room it is played in; null outside a room. */
  room_code: string | null
  surrendered_at: string | null
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

export interface PropositionVerdict extends PaceView {
  correct: boolean
  /** Only when correct. */
  layout?: Layout
}

export interface BotGameResult {
  ended_at: string
  /** In seconds. */
  score: number
  points: number
  surrendered: boolean
}

export interface BotStats {
  played: number
  solved: number
  surrendered: number
  best_seconds: number | null
  /** Over the last 5 finished Bot games. */
  average_points: number | null
  /** Up to 20 finished Bot games, oldest first. */
  recent: BotGameResult[]
}

export const getBotStats = (): Promise<BotStats> => apiRequest<BotStats>('/games/stats')
