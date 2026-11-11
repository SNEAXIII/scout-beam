import { apiRequest } from '@/app/lib/api'
import type { BeamColour } from '@/app/lib/colours'
import type { Layout, Piece } from '@/app/lib/pieces'

export type GameStatus = 'in_progress' | 'solved'

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
  /** Only once solved. */
  layout?: Layout
}

export const createGame = (): Promise<GameView> =>
  apiRequest<GameView>('/games', { method: 'POST' })

export const getGame = (id: string): Promise<GameView> => apiRequest<GameView>(`/games/${id}`)

/** Re-firing an entry answers the stored shot; a solved game answers 409. */
export const fireShot = (id: string, entry: number): Promise<Shot> =>
  apiRequest<Shot>(`/games/${id}/shots`, { method: 'POST', body: { entry } })

export interface PropositionVerdict {
  correct: boolean
  /** Only when correct. */
  layout?: Layout
}

/** A broken layout rule answers 422 naming it; a solved game answers 409. */
export const submitProposition = (
  id: string,
  pieces: readonly Piece[]
): Promise<PropositionVerdict> =>
  apiRequest<PropositionVerdict>(`/games/${id}/propositions`, {
    method: 'POST',
    body: { pieces },
  })
