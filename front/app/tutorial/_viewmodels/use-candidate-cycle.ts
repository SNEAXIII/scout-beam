import { useEffect, useState } from 'react'
import type { Shape } from '@/app/lib/pieces'

const PERIOD = 1500

/** The candidate shown filled, taking turns among `shapes`; the first stays under reduced motion. */
export function useCandidateCycle(shapes: readonly Shape[]): Shape | undefined {
  const key = shapes.join()
  const [cycle, setCycle] = useState({ key, tick: 0 })
  const tick = cycle.key === key ? cycle.tick : 0
  const count = shapes.length

  useEffect(() => {
    if (count < 2 || window.matchMedia?.('(prefers-reduced-motion: reduce)').matches) return
    const id = setInterval(
      () => setCycle((last) => ({ key, tick: (last.key === key ? last.tick : 0) + 1 })),
      PERIOD
    )
    return () => clearInterval(id)
  }, [key, count])

  return count ? shapes[tick % count] : undefined
}
