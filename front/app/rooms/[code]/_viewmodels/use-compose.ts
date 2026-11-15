import { useState } from 'react'
import { useI18n } from '@/app/i18n'
import { ApiError } from '@/app/lib/api'
import { clearDraft } from '@/app/lib/draft'
import type { Piece } from '@/app/lib/pieces'
import { hideLayout, type RoomView } from '@/app/services/rooms'

/** Where the layout being composed for a room is kept until it is hidden. */
export const composeKey = (code: string) => `room:${code}`

/** Hiding the caller's layout in a composing Duel; the room page polls on to the game. */
export function useCompose(code: string, onRoom: (room: RoomView) => void) {
  const { t } = useI18n()
  // The room does not say whose layout is hidden: a 409 after a reload tells it instead.
  const [hidden, setHidden] = useState(false)
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const hide = async (pieces: readonly Piece[]) => {
    setPending(true)
    setError(null)
    try {
      onRoom(await hideLayout(code, pieces))
    } catch (caught) {
      if (!(caught instanceof ApiError && caught.status === 409))
        return setError(caught instanceof Error ? caught.message : t.rooms.hideFailed)
    } finally {
      setPending(false)
    }
    setHidden(true)
    clearDraft(composeKey(code))
  }

  return { hidden, pending, error, hide }
}
