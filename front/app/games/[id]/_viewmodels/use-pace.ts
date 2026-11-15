import { useEffect, useState } from 'react'
import { FREE_SHOTS, allowance, cooldownMs, earnedAt } from '@/app/lib/pace'
import type { ServerMessage, SocketAction } from '@/app/services/game-socket'
import type { GameView, PaceView } from '@/app/services/games'

/** The server's last word on the pace, and when it arrived by this page's clock. */
interface Anchor {
  readonly elapsedMs: number
  readonly propositionInMs: number
  readonly at: number
}

export interface Pace {
  readonly shotsLeft: number
  readonly nextShotInMs: number
  /** From 0 to 1, towards the next earned shot. */
  readonly shotProgress: number
  readonly propositionInMs: number
  /** From 0 to 1 through the cooldown; 1 once a proposition may go. */
  readonly cooldownProgress: number
}

/** A room game's shot allowance and proposition cooldown, ticking locally from what the server
 * last said, so a skewed clock or a slow link never shifts it; null outside a room. */
export function usePace(game: GameView | null) {
  const [anchor, setAnchor] = useState<Anchor | null>(null)
  const [now, setNow] = useState(() => performance.now())
  const live = anchor !== null && game?.status === 'in_progress'

  useEffect(() => {
    if (!live) return
    const timer = setInterval(() => setNow(performance.now()), 100)
    return () => clearInterval(timer)
  }, [live])

  const anchorTo = (elapsedMs: number, propositionInMs: number) => {
    const at = performance.now()
    setNow(at)
    setAnchor({ elapsedMs, propositionInMs, at })
  }

  const since = anchor ? now - anchor.at : 0
  const elapsedMs = anchor ? anchor.elapsedMs + since : 0
  const propositionInMs = anchor ? Math.max(0, anchor.propositionInMs - since) : 0

  const sync = ({ elapsed_ms, proposition_in_ms }: PaceView) => {
    if (elapsed_ms !== undefined) anchorTo(elapsed_ms, proposition_in_ms ?? 0)
  }
  const receive = (message: ServerMessage) => {
    if (message.type === 'game') sync(message.game)
    if (message.type === 'shot' || message.type === 'verdict') sync(message)
  }

  /** A 409 says how long is left: the next earned shot, or the end of the cooldown. */
  const refused = (action: SocketAction, retryAfterMs: number) => {
    if (!anchor || !game) return
    if (action === 'fire')
      anchorTo(
        earnedAt(Math.max(1, game.shots.length - FREE_SHOTS + 1)) - retryAfterMs,
        propositionInMs
      )
    if (action === 'propose') anchorTo(elapsedMs, retryAfterMs)
  }

  const pace: Pace | null =
    anchor && game
      ? {
          ...allowance(elapsedMs, game.shots.length),
          propositionInMs,
          cooldownProgress: 1 - propositionInMs / cooldownMs(game.wrong_propositions),
        }
      : null
  return { pace, receive, refused }
}
