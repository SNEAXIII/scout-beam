export const WHEEL_RADIUS = 120

export type WheelRing = 'colour' | 'shape'
export type WheelSlice = { ring: WheelRing; index: number }

// Shapes inside: reaching one never crosses a colour, which is picked by passing over it.
export const WHEEL_DEAD_ZONE = 0.25
export const WHEEL_RINGS: Record<WheelRing, { inner: number; outer: number }> = {
  shape: { inner: WHEEL_DEAD_ZONE, outer: 0.7 },
  colour: { inner: 0.7, outer: 1 },
}

/** How many slices each ring holds right now. */
export type WheelCounts = Record<WheelRing, number>

const TURN = 2 * Math.PI

// Angle clockwise from straight up, with y pointing down.
const point = (radius: number, angle: number) => ({
  x: radius * Math.sin(angle),
  y: -radius * Math.cos(angle),
})

/** Past the edge only the direction counts: the outer ring's slice that way. */
export function wheelSliceAt(dx: number, dy: number, counts: WheelCounts): WheelSlice | null {
  const r = Math.hypot(dx, dy) / WHEEL_RADIUS
  if (r < WHEEL_DEAD_ZONE) return null
  const ring: WheelRing = r < WHEEL_RINGS.shape.outer ? 'shape' : 'colour'
  const slices = counts[ring]
  if (slices === 0) return null
  const angle = (Math.atan2(dx, -dy) + TURN) % TURN
  return { ring, index: Math.floor((angle * slices) / TURN + 0.5) % slices }
}

/** Each arc goes through its midpoint, so a lone slice still draws as a full ring. */
export function slicePath(ring: WheelRing, index: number, slices: number): string {
  const { inner, outer } = WHEEL_RINGS[ring]
  const [r0, r1] = [inner * WHEEL_RADIUS, outer * WHEEL_RADIUS]
  const step = TURN / slices
  const [start, mid, end] = [(index - 0.5) * step, index * step, (index + 0.5) * step]
  const arc = (r: number, sweep: 0 | 1, ...to: number[]) =>
    to.map((angle) => `A${r} ${r} 0 0 ${sweep} ${point(r, angle).x} ${point(r, angle).y}`).join('')
  const [a, c] = [point(r1, start), point(r0, end)]
  return `M${a.x} ${a.y}${arc(r1, 1, mid, end)}L${c.x} ${c.y}${arc(r0, 0, mid, start)}Z`
}

export function sliceCentre(
  ring: WheelRing,
  index: number,
  slices: number
): { x: number; y: number } {
  const { inner, outer } = WHEEL_RINGS[ring]
  return point(((inner + outer) / 2) * WHEEL_RADIUS, (index * TURN) / slices)
}

/** The pointer's offset pulled back onto the wheel's edge once past it. */
export function clampToWheel(dx: number, dy: number): { x: number; y: number } {
  const scale = Math.min(1, WHEEL_RADIUS / (Math.hypot(dx, dy) || 1))
  return { x: dx * scale, y: dy * scale }
}
