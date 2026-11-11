import { apiRequest } from '@/app/lib/api'
import { saveTokens, type TokenPair } from '@/app/lib/session'

export async function devLogin(login: string): Promise<void> {
  const tokens = await apiRequest<TokenPair>('/auth/dev-login', {
    method: 'POST',
    body: { login },
    anonymous: true,
  })
  saveTokens(tokens)
}
