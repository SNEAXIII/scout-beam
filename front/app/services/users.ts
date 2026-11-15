import { apiRequest } from '@/app/lib/api'
import type { UserProfile } from '@/app/services/auth'

/** The session is untouched: the tokens carry the account id, not the login. */
export const rename = (login: string) =>
  apiRequest<UserProfile>('/users/me', { method: 'PATCH', body: { login } })
