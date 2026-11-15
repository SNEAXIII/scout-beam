export const API_URL = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:8002'

export type ErrorParams = Record<string, string | number>

/** `code` names the reason for `errorText` to translate; `message` is the API's English fallback. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly code = '',
    readonly params: ErrorParams = {},
    options?: ErrorOptions
  ) {
    super(message, options)
  }
}

/** An error body, over HTTP or the game socket. */
export interface ErrorBody {
  code?: string
  params?: ErrorParams
  detail?: unknown
}

export const bodyError = (status: number, body: ErrorBody | null) =>
  new ApiError(
    status,
    detailMessage(body?.detail) ?? `Error ${status}`,
    body?.code ?? 'http_error',
    body?.params ?? { status }
  )

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
    throw new ApiError(0, 'Server unreachable', 'unreachable', {}, { cause })
  })
}

/** The API's `detail`: a message, or a list of `{msg}` for a failed validation. */
function detailMessage(detail: unknown): string | null {
  if (typeof detail === 'string') return detail
  if (Array.isArray(detail)) {
    const messages = detail
      .map((item: { msg?: unknown }) => item.msg)
      .filter((msg): msg is string => typeof msg === 'string')
    if (messages.length > 0) return messages.join(' ')
  }
  return null
}

const responseError = async (response: Response) =>
  bodyError(response.status, (await response.json().catch(() => null)) as ErrorBody | null)

/** New session cookies from the refresh cookie; false when it is missing or refused, any other failure throws. */
export async function refreshSession(): Promise<boolean> {
  const response = await send('/auth/refresh', { method: 'POST' })
  if (response.status === 401) return false
  if (!response.ok) throw await responseError(response)
  return true
}

/** Back to the page left once signed in again. */
function toLogin(): never {
  const { pathname, search } = window.location
  if (pathname !== '/login')
    window.location.assign(`/login?next=${encodeURIComponent(pathname + search)}`)
  throw new ApiError(401, 'Session expired', 'session_expired')
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
  if (!response.ok) throw await responseError(response)
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}
