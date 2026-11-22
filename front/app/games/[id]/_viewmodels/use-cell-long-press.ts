import { useEffect, useRef } from 'react'
import { LONG_PRESS_MS, SLOP_PX, swallowNextClick } from './use-long-press'

type Point = { readonly x: number; readonly y: number }

/** Handlers for the cell grid: a touch long press on an empty cell calls `onLongPress` where it lands. */
export function useCellLongPress(onLongPress: (at: Point) => void) {
  const press = useRef<(Point & { timer: number }) | null>(null)
  const dropGuard = useRef<() => void>(undefined)
  const cancelPress = () => window.clearTimeout(press.current?.timer)
  useEffect(
    () => () => {
      window.clearTimeout(press.current?.timer)
      dropGuard.current?.()
    },
    []
  )

  return {
    onPointerDown: (event: React.PointerEvent) => {
      cancelPress()
      press.current = null
      const target = event.target as HTMLElement
      if (event.pointerType !== 'touch' || target.dataset.piece) return
      const at = { x: event.clientX, y: event.clientY }
      const timer = window.setTimeout(() => {
        dropGuard.current?.()
        dropGuard.current = swallowNextClick()
        onLongPress(at)
      }, LONG_PRESS_MS)
      press.current = { ...at, timer }
    },
    onPointerMove: ({ clientX, clientY }: React.PointerEvent) => {
      const start = press.current
      if (start && Math.hypot(clientX - start.x, clientY - start.y) > SLOP_PX) cancelPress()
    },
    onPointerUp: cancelPress,
    onPointerCancel: cancelPress,
  }
}
