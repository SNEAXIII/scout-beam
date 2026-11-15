import { apiRequest, refreshSession } from '@/app/lib/api'
import { nextPath } from '@/app/lib/utils'

/** `GET /auth/session` and the body of every sign-in: who is signed in, never their tokens. */
export interface UserProfile {
  login: string
  role: 'super_admin' | 'admin' | 'user'
  guest: boolean
  last_login_date: string | null
  created_at: string
}

export type ProviderName = 'discord' | 'google'

export interface ProviderLink {
  name: ProviderName
  /** Carries everything but `state`, which the front appends. */
  authorize_url: string
}

export interface Providers {
  dev_login: boolean
  providers: ProviderLink[]
}

/** The API sets the session cookies; a sign-in's 401 is an answer to show, not a bounce to /login. */
const signIn = (path: string, body?: unknown) =>
  apiRequest<UserProfile>(path, { method: 'POST', body, noRedirect: true })

/** The API ignores an expired access cookie: renewed first, so a guest is converted, not left behind. */
async function convertingSignIn(path: string, body: unknown): Promise<UserProfile> {
  await refreshSession().catch(() => false)
  return signIn(path, body)
}

export const login = (login: string, password: string) => signIn('/auth/login', { login, password })

export const register = (login: string, password: string) =>
  convertingSignIn('/auth/register', { login, password })

export const guestLogin = () => signIn('/auth/guest')

export const oauthLogin = (provider: ProviderName, code: string) =>
  convertingSignIn(`/auth/${provider}`, { code })

export const devLogin = (login: string) => signIn('/auth/dev-login', { login })

export const getSession = () => apiRequest<UserProfile>('/auth/session')

export const logout = () =>
  apiRequest<undefined>('/auth/logout', { method: 'POST', noRedirect: true })

export const listProviders = () => apiRequest<Providers>('/auth/providers', { noRedirect: true })

export const isProviderName = (name: string): name is ProviderName =>
  name === 'discord' || name === 'google'

const OAUTH_STATE_KEY = 'scout-beam.oauth-state'
const OAUTH_NEXT_KEY = 'scout-beam.oauth-next'

/** Off to the provider, with a `state` the callback checks so a forged return is refused. */
export function beginOAuth(provider: ProviderLink, next: string | null): void {
  const state = crypto.randomUUID()
  try {
    sessionStorage.setItem(OAUTH_STATE_KEY, state)
    sessionStorage.setItem(OAUTH_NEXT_KEY, nextPath(next))
  } catch {
    // Without storage the callback finds no state and refuses: nothing is sent.
  }
  window.location.assign(`${provider.authorize_url}&state=${encodeURIComponent(state)}`)
}

/** A stored value, read once: a reloaded callback cannot replay it. */
function take(key: string): string | null {
  try {
    const value = sessionStorage.getItem(key)
    sessionStorage.removeItem(key)
    return value
  } catch {
    return null
  }
}

export const takeOAuthState = () => take(OAUTH_STATE_KEY)

/** Where the sign-in that started `beginOAuth` meant to land. */
export const takeOAuthNext = () => nextPath(take(OAUTH_NEXT_KEY))
