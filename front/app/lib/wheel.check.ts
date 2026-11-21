import assert from 'node:assert/strict'
import { WHEEL_RADIUS, WHEEL_RINGS, sliceCentre, slicePath, wheelSliceAt } from './wheel.ts'

const R = WHEEL_RADIUS
const at = (radius: number, degrees: number) => {
  const a = (degrees * Math.PI) / 180
  return wheelSliceAt(radius * R * Math.sin(a), -radius * R * Math.cos(a))
}

assert.equal(at(0.2, 0), null, 'dead zone')
assert.equal(at(0.24, 0), null, 'dead zone edge')
assert.equal(at(1.01, 0), null, 'beyond the radius')
assert.deepEqual(at(0.26, 0), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.54, 0), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.56, 0), { ring: 'shape', index: 0 })
assert.deepEqual(at(0.99, 0), { ring: 'shape', index: 0 })

assert.deepEqual(at(0.4, 44), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.4, 46), { ring: 'colour', index: 1 })
assert.deepEqual(at(0.4, -44), { ring: 'colour', index: 0 })
assert.deepEqual(at(0.4, -46), { ring: 'colour', index: 3 })
assert.deepEqual(at(0.8, 35), { ring: 'shape', index: 0 })
assert.deepEqual(at(0.8, 37), { ring: 'shape', index: 1 })
assert.deepEqual(at(0.8, -37), { ring: 'shape', index: 4 })

for (const ring of ['colour', 'shape'] as const)
  for (let index = 0; index < WHEEL_RINGS[ring].slices; index += 1) {
    const { x, y } = sliceCentre(ring, index)
    assert.deepEqual(wheelSliceAt(x, y), { ring, index }, `${ring} ${index} centre`)
    const d = slicePath(ring, index)
    assert.ok(d.startsWith('M') && d.endsWith('Z') && !d.includes('NaN'), `${ring} ${index} path`)
  }

console.log('wheel ok')
