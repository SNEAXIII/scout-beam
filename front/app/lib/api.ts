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
  /** A sign-in's 401 is an answer to show, not an expired session: no refresh, no /login. */
  noRedirect?: boolean
}

/** The session lives in `HttpOnly` cookies the browser sends itself: no script ever sees a token. */
const send = (path: string, { method = 'GET', body }: RequestOptions) => {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  return fetch(`${API_URL}${path}`, {
    method,
    headers,
    credentials: 'include',
    body: body === undefined ? undefined : JSON.stringify(body),
  }).catch((cause: unknown) => {
    throw new ApiError(0, 'Serveur injoignable', { cause })
  })
}

/** The API's `detail`: a message, or a list of `{msg}` for a failed validation. */
export function detailMessage(detail: unknown): string | null {
  if (typeof detail === 'string') return detail
  if (Array.isArray(detail)) {
    const messages = detail
      .map((item: { msg?: unknown }) => item.msg)
      .filter((msg): msg is string => typeof msg === 'string')
    if (messages.length > 0) return messages.join(' ')
  }
  return null
}

async function errorMessage(response: Response): Promise<string> {
  const payload = (await response.json().catch(() => null)) as { detail?: unknown } | null
  return detailMessage(payload?.detail) ?? `Erreur ${response.status}`
}

/** New session cookies from the refresh cookie; false when it is missing or refused, any other failure throws. */
export async function refreshSession(): Promise<boolean> {
  const response = await send('/auth/refresh', { method: 'POST' })
  if (response.status === 401) return false
  if (!response.ok) throw new ApiError(response.status, await errorMessage(response))
  return true
}

function toLogin(): never {
  if (window.location.pathname !== '/login') window.location.assign('/login')
  throw new ApiError(401, 'Session expirée')
}

/** For a refused socket: new cookies, or off to /login, which throws a 401 `ApiError`. */
export async function renewSession(): Promise<void> {
  if (!(await refreshSession())) toLogin()
}

export async function apiRequest<T>(path: string, options: RequestOptions = {}): Promise<T> {
  let response = await send(path, options)
  if (response.status === 401 && !options.noRedirect) {
    if (!(await refreshSession())) toLogin()
    response = await send(path, options)
    if (response.status === 401) toLogin()
  }
  if (!response.ok) throw new ApiError(response.status, await errorMessage(response))
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}
