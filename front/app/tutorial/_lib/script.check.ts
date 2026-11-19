import assert from 'node:assert/strict'
import { traceBeam, type Trace } from '../../lib/trace.ts'
import type { Layout } from '../../lib/pieces.ts'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from './script.ts'

const shots = TUTORIAL_STEPS.flatMap((step) => (step.kind === 'shot' ? [step] : []))
const fire = (layout: Layout, entry: number): Trace => {
  const trace = traceBeam(layout, entry)
  assert.ok(trace, `entry ${entry} never leaves`)
  return trace
}
const answer = (layout: Layout, entry: number) => {
  const { exit, colour } = fire(layout, entry)
  return `${exit} ${colour}`
}
const hidden = shots.map(({ entry }) => fire(TUTORIAL_LAYOUT, entry))

assert.deepEqual(
  TUTORIAL_STEPS.map((step) => step.kind),
  ['shot', 'shot', 'shot', 'shot', 'shot', 'shot', 'draft', 'propose']
)

assert.equal(hidden[0].colour, 'none', 'step 1 touches nothing')
assert.equal(hidden[1].exit, hidden[1].entry, 'step 2 is a Return')
assert.equal(hidden[1].colour, 'red', 'step 2 comes off the Square')
const [a, turn, b] = hidden[2].points
assert.equal(hidden[2].points.length, 3, 'step 3 turns once')
const dot = (turn.x - a.x) * (b.x - turn.x) + (turn.y - a.y) * (b.y - turn.y)
assert.ok(dot === 0, 'step 3 turns 90°')
assert.equal(hidden[2].colour, 'yellow', 'step 3 comes off the Corner')
assert.equal(hidden[5].colour, 'orange', 'step 6 touches both pieces')

const [square] = TUTORIAL_LAYOUT.pieces
const candidates = shots.flatMap((shot) => shot.candidates ?? [])
assert.deepEqual(candidates.map((piece) => piece.shape).sort(), ['corner', 'diamond', 'spike'])

for (const candidate of candidates) {
  const layout: Layout = { pieces: [square, candidate] }
  const out = shots.findIndex((shot) => shot.eliminated?.includes(candidate.shape))
  if (candidate.shape === 'corner') assert.equal(out, -1, 'the Corner is never eliminated')
  const kept = out === -1 ? shots : shots.slice(0, out)
  for (const { entry } of kept)
    assert.equal(
      answer(layout, entry),
      answer(TUTORIAL_LAYOUT, entry),
      `${candidate.shape} on ${entry}`
    )
  if (out !== -1) {
    const { entry } = shots[out]
    assert.notEqual(
      answer(layout, entry),
      answer(TUTORIAL_LAYOUT, entry),
      `${candidate.shape} out on ${entry}`
    )
  }
}

console.log('tutorial script ok')
