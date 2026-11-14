import { apiRequest, freshAccessToken } from '@/app/lib/api'
import { isGuestSession, markGuest, saveTokens, type TokenPair } from '@/app/lib/session'

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

interface SignInOptions {
  /** The account signed into is a guest. */
  asGuest?: boolean
  /** The route turns a guest bearer into the account: register and the provider sign-ins. */
  convertsGuest?: boolean
}

/** A sign-in never bounces to /login: its 401 is an answer to show, not an expired session. */
async function signIn(
  path: string,
  body?: unknown,
  { asGuest = false, convertsGuest = false }: SignInOptions = {}
): Promise<void> {
  // The API ignores an expired bearer: a stale one would make a new account and lose the guest's games.
  const bearer = convertsGuest && isGuestSession() ? await freshAccessToken() : null
  const tokens = await apiRequest<TokenPair>(path, {
    method: 'POST',
    body,
    anonymous: true,
    bearer,
  })
  saveTokens(tokens)
  markGuest(asGuest)
}

export const login = (login: string, password: string) => signIn('/auth/login', { login, password })

export const register = (login: string, password: string) =>
  signIn('/auth/register', { login, password }, { convertsGuest: true })

export const guestLogin = () => signIn('/auth/guest', undefined, { asGuest: true })

export const oauthLogin = (provider: ProviderName, code: string) =>
  signIn(`/auth/${provider}`, { code }, { convertsGuest: true })

export const devLogin = (login: string) => signIn('/auth/dev-login', { login })

export const listProviders = () => apiRequest<Providers>('/auth/providers', { anonymous: true })

export const isProviderName = (name: string): name is ProviderName =>
  name === 'discord' || name === 'google'

const OAUTH_STATE_KEY = 'scout-beam.oauth-state'

/** Off to the provider, with a `state` the callback checks so a forged return is refused. */
export function beginOAuth(provider: ProviderLink): void {
  const state = crypto.randomUUID()
  try {
    sessionStorage.setItem(OAUTH_STATE_KEY, state)
  } catch {
    // Without storage the callback finds no state and refuses: nothing is sent.
  }
  window.location.assign(`${provider.authorize_url}&state=${encodeURIComponent(state)}`)
}

/** The stored `state`, read once: a reloaded callback cannot replay it. */
export function takeOAuthState(): string | null {
  try {
    const state = sessionStorage.getItem(OAUTH_STATE_KEY)
    sessionStorage.removeItem(OAUTH_STATE_KEY)
    return state
  } catch {
    return null
  }
}
