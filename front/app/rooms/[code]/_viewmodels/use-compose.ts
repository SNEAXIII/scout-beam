import { useState } from 'react'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { clearDraft } from '@/app/lib/draft'
import type { Piece } from '@/app/lib/pieces'
import { getRoom, hideLayout, type RoomView } from '@/app/services/rooms'

/** Where the layout being composed for a room is kept until it is hidden. */
export const composeKey = (code: string) => `room:${code}`

/** Hiding the caller's layout in a composing Duel; the room says once it is, reloads included. */
export function useCompose(room: RoomView, onRoom: (room: RoomView) => void) {
  const { t } = useI18n()
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const hide = async (pieces: readonly Piece[]) => {
    setPending(true)
    setError(null)
    try {
      onRoom(await hideLayout(room.code, pieces))
      clearDraft(composeKey(room.code))
    } catch (caught) {
      // Hidden already, from another tab: the room shows it.
      if (caught instanceof ApiError && caught.status === 409) onRoom(await getRoom(room.code))
      else setError(caught instanceof Error ? caught.message : t.rooms.hideFailed)
    } finally {
      setPending(false)
    }
  }

  return { hidden: room.hidden_layout?.pieces ?? null, pending, error, hide }
}
