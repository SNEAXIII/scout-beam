import { apiRequest } from '@/app/lib/api'
import type { Piece } from '@/app/lib/pieces'

export type RoomMode = 'race' | 'duel'
export type RoomStatus = 'waiting' | 'composing' | 'playing' | 'finished'

/** A non-member sees only the code, mode, status and host. */
export interface RoomView {
  code: string
  mode: RoomMode
  status: RoomStatus
  host: string
  opponent?: string
  winner?: string
  /** The caller's game, once it exists. */
  game_id?: string
  /** A playing Duel only. */
  last_actor?: string
  retry_after_ms?: number
}

/** No 0, O, 1 or I: nothing to misread when a code is dictated. */
export const ROOM_CODE_ALPHABET = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789'

export const isRoomCode = (code: string) =>
  code.length === 6 && [...code].every((char) => ROOM_CODE_ALPHABET.includes(char))

export const createRoom = (mode: RoomMode) =>
  apiRequest<RoomView>('/rooms', { method: 'POST', body: { mode } })

export const getRoom = (code: string) => apiRequest<RoomView>(`/rooms/${code}`)

export const joinRoom = (code: string) =>
  apiRequest<RoomView>(`/rooms/${code}/join`, { method: 'POST' })

/** A composing Duel's layout for the opponent: 409 once hidden already, 422 naming a broken rule. */
export const hideLayout = (code: string, pieces: readonly Piece[]) =>
  apiRequest<RoomView>(`/rooms/${code}/layout`, { method: 'POST', body: { pieces } })
