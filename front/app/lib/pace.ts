// Mirrors `api/src/engine/pace.rs`, the source of truth: change both together.
export const FREE_SHOTS = 15
const FIRST_SHOT_MS = 5000
const SHOT_STEP_MS = 500
const COOLDOWN_MS = 5000
const COOLDOWN_PER_WRONG_MS = 1000
const COOLDOWN_CAP_MS = 10_000

/** When the k-th earned shot arrives: `5 s × k + 0.25 s × k × (k − 1)`. */
export const earnedAt = (k: number) => FIRST_SHOT_MS * k + (SHOT_STEP_MS * k * (k - 1)) / 2

/** The shots left `fired` shots in, and how far the next earned one has come, from 0 to 1. */
export function allowance(elapsedMs: number, fired: number) {
  let earned = 0
  while (earnedAt(earned + 1) <= elapsedMs) earned += 1
  const from = earnedAt(earned)
  const to = earnedAt(earned + 1)
  return {
    shotsLeft: FREE_SHOTS + earned - fired,
    nextShotInMs: to - elapsedMs,
    shotProgress: (elapsedMs - from) / (to - from),
  }
}

/** The wait after a proposition, these wrong ones counted: 5 s after the first. */
export const cooldownMs = (wrongPropositions: number) =>
  Math.min(
    COOLDOWN_CAP_MS,
    COOLDOWN_MS + COOLDOWN_PER_WRONG_MS * Math.max(0, wrongPropositions - 1)
  )
