import { clearTokens, getAccessToken, getRefreshToken, saveTokens, type TokenPair } from './session'

export const API_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8002'

export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
    options?: ErrorOptions
  ) {
    super(message, options)
  }
}

interface RequestOptions {
  method?: 'GET' | 'POST'
  body?: unknown
  /** Sign-in calls carry no token and must not bounce to /login on a 401. */
  anonymous?: boolean
}

const send = (path: string, { method = 'GET', body, anonymous }: RequestOptions) => {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  const token = anonymous ? null : getAccessToken()
  if (token) headers.Authorization = `Bearer ${token}`
  return fetch(`${API_URL}${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
  }).catch((cause: unknown) => {
    throw new ApiError(0, 'Serveur injoignable', { cause })
  })
}

/** The API answers `{detail}`: a message, or a list of `{msg}` for a failed validation. */
async function errorMessage(response: Response): Promise<string> {
  const payload = (await response.json().catch(() => null)) as { detail?: unknown } | null
  const detail = payload?.detail
  if (typeof detail === 'string') return detail
  if (Array.isArray(detail)) {
    const messages = detail
      .map((item: { msg?: unknown }) => item.msg)
      .filter((msg): msg is string => typeof msg === 'string')
    if (messages.length > 0) return messages.join(' ')
  }
  return `Erreur ${response.status}`
}

/** False only when the refresh token is missing or refused; any other failure throws. */
async function refreshTokens(): Promise<boolean> {
  const refreshToken = getRefreshToken()
  if (!refreshToken) return false
  const response = await send('/auth/refresh', {
    method: 'POST',
    body: { refresh_token: refreshToken },
    anonymous: true,
  })
  if (response.status === 401) return false
  if (!response.ok) throw new ApiError(response.status, await errorMessage(response))
  saveTokens((await response.json()) as TokenPair)
  return true
}

function toLogin(): never {
  clearTokens()
  if (window.location.pathname !== '/login') window.location.assign('/login')
  throw new ApiError(401, 'Session expirée')
}

export async function apiRequest<T>(path: string, options: RequestOptions = {}): Promise<T> {
  let response = await send(path, options)
  if (response.status === 401 && !options.anonymous) {
    if (!(await refreshTokens())) toLogin()
    response = await send(path, options)
    if (response.status === 401) toLogin()
  }
  if (!response.ok) throw new ApiError(response.status, await errorMessage(response))
  return (await response.json()) as T
}
