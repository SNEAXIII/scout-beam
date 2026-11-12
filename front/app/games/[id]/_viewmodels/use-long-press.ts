import { useEffect, useRef } from 'react'
import type { Shape } from '@/app/lib/pieces'

const LONG_PRESS_MS = 500
const SLOP_PX = 10
const CLICK_GUARD_MS = 400

/** Swallows the click a browser may synthesise when a long press lifts over the emptied cell. */
function swallowNextClick(): () => void {
  let expiry: number | undefined
  const expire = () => {
    window.clearTimeout(expiry)
    expiry = window.setTimeout(drop, CLICK_GUARD_MS)
  }
  const swallow = (event: Event) => {
    event.stopPropagation()
    event.preventDefault()
    drop()
  }
  // A new press or a key drops the guard at once, so only the lift's own click is swallowed.
  const listeners = Object.entries({
    click: swallow,
    pointerdown: drop,
    keydown: drop,
    pointerup: expire,
    pointercancel: expire,
  })
  function drop() {
    window.clearTimeout(expiry)
    for (const [type, listener] of listeners) window.removeEventListener(type, listener, true)
  }
  for (const [type, listener] of listeners) window.addEventListener(type, listener, true)
  return drop
}

/** Piece handlers calling `onRemove` on a right click or a touch long press, once per press. */
export function useLongPress(onRemove?: (shape: Shape) => void) {
  // `done` keeps the contextmenu Android fires on a long press from removing the piece twice.
  const press = useRef<{ timer: number; x: number; y: number; done: boolean } | null>(null)
  const dropGuard = useRef<() => void>(undefined)
  const cancelPress = () => window.clearTimeout(press.current?.timer)
  useEffect(
    () => () => {
      window.clearTimeout(press.current?.timer)
      dropGuard.current?.()
    },
    []
  )

  const removeOnce = (shape: Shape, event?: React.MouseEvent) => {
    event?.preventDefault()
    if (press.current?.done) return
    cancelPress()
    if (press.current) {
      press.current.done = true
      dropGuard.current?.()
      dropGuard.current = swallowNextClick()
    }
    onRemove?.(shape)
  }

  return (shape: Shape) => ({
    onContextMenu: (event: React.MouseEvent) => removeOnce(shape, event),
    onPointerDown: ({ pointerType, clientX, clientY }: React.PointerEvent) => {
      cancelPress()
      press.current = null
      if (pointerType !== 'touch') return
      const timer = window.setTimeout(() => removeOnce(shape), LONG_PRESS_MS)
      press.current = { timer, x: clientX, y: clientY, done: false }
    },
    onPointerMove: ({ clientX, clientY }: React.PointerEvent) => {
      const start = press.current
      if (start && Math.hypot(clientX - start.x, clientY - start.y) > SLOP_PX) cancelPress()
    },
    onPointerUp: cancelPress,
    onPointerCancel: cancelPress,
  })
}
