import { API_URL, ApiError, renewSession } from '@/app/lib/api'
import type { Piece } from '@/app/lib/pieces'
import { getAccessToken } from '@/app/lib/session'
import type { GameView, PropositionVerdict, Shot } from './games'

/** The client message an error answers; `connect` errors close the socket. */
export type SocketAction = 'connect' | 'fire' | 'propose' | 'surrender' | 'message'

/** One answer per client message, in order, plus what the player's other tabs changed. */
export type ServerMessage =
  | { type: 'game'; game: GameView }
  | { type: 'shot'; shot: Shot; replayed: boolean }
  | ({ type: 'verdict' } & PropositionVerdict)
  | { type: 'error'; action: SocketAction; status: number; detail: unknown }

export type ClientMessage =
  | { type: 'fire'; entry: number }
  | { type: 'propose'; pieces: readonly Piece[] }
  | { type: 'surrender' }

export interface GameSocketHandlers {
  onMessage: (message: ServerMessage) => void
  /** True once the game's snapshot arrived, false as soon as the socket drops. */
  onConnected: (connected: boolean) => void
}

export interface GameSocket {
  /** False when the socket is not ready: the message was not sent. */
  send: (message: ClientMessage) => boolean
  close: () => void
}

const SOCKET_URL = API_URL.replace(/^http/, 'ws')
const RETRY_MS = [500, 1000, 2000, 5000]

/** The game's socket, reopened when it drops; a 401 renews the session first, a 404 is final. */
export function openGameSocket(id: string, handlers: GameSocketHandlers): GameSocket {
  let socket: WebSocket | null = null
  let ready = false
  let closed = false
  let attempts = 0
  let refusal: number | null = null
  let timer: ReturnType<typeof setTimeout> | undefined

  const retry = () => {
    timer = setTimeout(connect, RETRY_MS[Math.min(attempts, RETRY_MS.length - 1)])
    attempts += 1
  }

  const reopen = (status: number | null) => {
    if (closed || status === 404) return
    if (status !== 401) return retry()
    renewSession().then(connect, (caught: unknown) => {
      // A 401 here means the player is on their way to /login.
      if (!(caught instanceof ApiError && caught.status === 401)) retry()
    })
  }

  function connect() {
    if (closed) return
    const opened = new WebSocket(`${SOCKET_URL}/games/${id}/ws`)
    socket = opened
    opened.onopen = () => opened.send(JSON.stringify({ type: 'auth', token: getAccessToken() }))
    opened.onmessage = (event: MessageEvent<string>) => {
      const message = JSON.parse(event.data) as ServerMessage
      if (message.type === 'error' && message.action === 'connect') refusal = message.status
      if (message.type === 'game' && !ready) {
        ready = true
        attempts = 0
        handlers.onConnected(true)
      }
      handlers.onMessage(message)
    }
    opened.onclose = () => {
      if (socket !== opened) return
      socket = null
      ready = false
      handlers.onConnected(false)
      const status = refusal
      refusal = null
      reopen(status)
    }
  }

  connect()
  return {
    send: (message) => {
      if (!ready || socket?.readyState !== WebSocket.OPEN) return false
      socket.send(JSON.stringify(message))
      return true
    },
    close: () => {
      closed = true
      clearTimeout(timer)
      socket?.close()
      socket = null
    },
  }
}
