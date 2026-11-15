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
