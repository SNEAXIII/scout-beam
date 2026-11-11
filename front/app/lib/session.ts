const ACCESS_KEY = 'scout-beam.access-token'
const REFRESH_KEY = 'scout-beam.refresh-token'

export interface TokenPair {
  access_token: string
  refresh_token: string
}

const read = (key: string): string | null => {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

export const getAccessToken = (): string | null => read(ACCESS_KEY)
export const getRefreshToken = (): string | null => read(REFRESH_KEY)

/** Storage can be blocked (private mode, disabled site data): the session then lasts the page. */
export function saveTokens(tokens: TokenPair): void {
  try {
    localStorage.setItem(ACCESS_KEY, tokens.access_token)
    localStorage.setItem(REFRESH_KEY, tokens.refresh_token)
  } catch {
    // Nothing to fall back to: without a stored token the player lands back on /login.
  }
}

export function clearTokens(): void {
  try {
    localStorage.removeItem(ACCESS_KEY)
    localStorage.removeItem(REFRESH_KEY)
  } catch {
    // Unreadable storage holds no token either.
  }
}
