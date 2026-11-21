export const WHEEL_RADIUS = 120

export type WheelRing = 'colour' | 'shape'
export type WheelSlice = { ring: WheelRing; index: number }

export const WHEEL_DEAD_ZONE = 0.25
export const WHEEL_RINGS: Record<WheelRing, { inner: number; outer: number; slices: number }> = {
  colour: { inner: WHEEL_DEAD_ZONE, outer: 0.55, slices: 4 },
  shape: { inner: 0.55, outer: 1, slices: 5 },
}

const TURN = 2 * Math.PI
const step = (ring: WheelRing) => TURN / WHEEL_RINGS[ring].slices

// Angle clockwise from straight up, with y pointing down.
const point = (radius: number, angle: number) => ({
  x: radius * Math.sin(angle),
  y: -radius * Math.cos(angle),
})

export function wheelSliceAt(dx: number, dy: number): WheelSlice | null {
  const r = Math.hypot(dx, dy) / WHEEL_RADIUS
  if (r < WHEEL_DEAD_ZONE || r > 1) return null
  const ring: WheelRing = r < WHEEL_RINGS.colour.outer ? 'colour' : 'shape'
  const angle = (Math.atan2(dx, -dy) + TURN) % TURN
  const { slices } = WHEEL_RINGS[ring]
  return { ring, index: Math.floor(angle / step(ring) + 0.5) % slices }
}

export function slicePath(ring: WheelRing, index: number): string {
  const { inner, outer } = WHEEL_RINGS[ring]
  const [r0, r1] = [inner * WHEEL_RADIUS, outer * WHEEL_RADIUS]
  const start = (index - 0.5) * step(ring)
  const end = start + step(ring)
  const [a, b, c, d] = [point(r1, start), point(r1, end), point(r0, end), point(r0, start)]
  return `M${a.x} ${a.y}A${r1} ${r1} 0 0 1 ${b.x} ${b.y}L${c.x} ${c.y}A${r0} ${r0} 0 0 0 ${d.x} ${d.y}Z`
}

export function sliceCentre(ring: WheelRing, index: number): { x: number; y: number } {
  const { inner, outer } = WHEEL_RINGS[ring]
  return point(((inner + outer) / 2) * WHEEL_RADIUS, index * step(ring))
}
