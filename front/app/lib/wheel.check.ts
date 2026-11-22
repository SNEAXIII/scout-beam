import assert from 'node:assert/strict'
import {
  WHEEL_RADIUS,
  clampToWheel,
  sliceCentre,
  slicePath,
  wheelSliceAt,
  type WheelCounts,
} from './wheel.ts'

const R = WHEEL_RADIUS
const FULL: WheelCounts = { colour: 4, shape: 5 }
const at = (radius: number, degrees: number, counts = FULL) => {
  const a = (degrees * Math.PI) / 180
  return wheelSliceAt(radius * R * Math.sin(a), -radius * R * Math.cos(a), counts)
}

assert.equal(at(0.2, 0), null, 'dead zone')
assert.equal(at(0.24, 0), null, 'dead zone edge')
assert.deepEqual(at(3, 0), { ring: 'colour', index: 0 }, 'beyond the radius')
assert.deepEqual(at(3, -46), { ring: 'colour', index: 3 }, 'beyond the radius')
assert.deepEqual(at(0.26, 0), { ring: 'shape', index: 0 })
assert.deepEqual(at(0.69, 0), { ring: 'shape', index: 0 })
assert.deepEqual(at(0.71, 0), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.99, 0), { ring: 'colour', index: 0 })

assert.deepEqual(at(0.85, 44), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.85, 46), { ring: 'colour', index: 1 })
assert.deepEqual(at(0.85, -44), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.85, -46), { ring: 'colour', index: 3 })
assert.deepEqual(at(0.5, 35), { ring: 'shape', index: 0 })
assert.deepEqual(at(0.5, 37), { ring: 'shape', index: 1 })
assert.deepEqual(at(0.5, -37), { ring: 'shape', index: 4 })

assert.deepEqual(at(0.5, 89, { colour: 4, shape: 2 }), { ring: 'shape', index: 0 }, 'two shapes')
assert.deepEqual(at(0.5, 91, { colour: 4, shape: 2 }), { ring: 'shape', index: 1 }, 'two shapes')
assert.deepEqual(at(0.5, 180, { colour: 4, shape: 1 }), { ring: 'shape', index: 0 }, 'lone shape')
assert.equal(at(0.5, 0, { colour: 4, shape: 0 }), null, 'no shape left')

for (const shapes of [1, 2, 3, 4, 5])
  for (const ring of ['colour', 'shape'] as const) {
    const counts = { ...FULL, shape: shapes }
    for (let index = 0; index < counts[ring]; index += 1) {
      const { x, y } = sliceCentre(ring, index, counts[ring])
      assert.deepEqual(wheelSliceAt(x, y, counts), { ring, index }, `${ring} ${index} centre`)
      const d = slicePath(ring, index, counts[ring])
      assert.ok(d.startsWith('M') && d.endsWith('Z') && !d.includes('NaN'), `${ring} ${index} path`)
    }
  }

assert.deepEqual(clampToWheel(3 * R, 0), { x: R, y: 0 }, 'clamped onto the edge')
assert.deepEqual(clampToWheel(10, -20), { x: 10, y: -20 }, 'inside, untouched')

console.log('wheel ok')
