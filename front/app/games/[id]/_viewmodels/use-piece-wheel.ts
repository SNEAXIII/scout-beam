import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { KEYMAPS } from '@/app/lib/keyboard'
import { freeShapes, PIECE_COLOURS } from '@/app/lib/pieces'
import { clampToWheel, WHEEL_RADIUS, wheelSliceAt, type WheelSlice } from '@/app/lib/wheel'
import { typing, type Draft } from './use-draft'
import { LONG_PRESS_MS, SLOP_PX, swallowNextClick } from './use-long-press'

type Point = { readonly x: number; readonly y: number }

const clamp = (value: number, size: number) =>
  Math.max(WHEEL_RADIUS, Math.min(value, size - WHEEL_RADIUS))

const HIDE_CURSOR = ['cursor-none', '[&_*]:cursor-none!']

const sameSlice = (a: WheelSlice | null, b: WheelSlice | null) =>
  a?.ring === b?.ring && a?.index === b?.index

/** The piece wheel over `board`: held open with the wheel key, or a touch long press on an empty cell. */
export function usePieceWheel(draft: Draft, board: React.RefObject<HTMLElement | null>) {
  const [open, setOpen] = useState<Point | null>(null)
  const [hovered, setHovered] = useState<WheelSlice | null>(null)
  const [cursor, setCursor] = useState<Point | null>(null)
  // Refs as well: the window listeners read them between renders.
  const openRef = useRef<Point | null>(null)
  const hoveredRef = useRef<WheelSlice | null>(null)
  const pointer = useRef<Point | null>(null)
  const press = useRef<(Point & { timer: number }) | null>(null)
  const dropGuard = useRef<() => void>(undefined)

  const show = useCallback((at: Point | null) => {
    const centre = at && {
      x: clamp(at.x, document.documentElement.clientWidth),
      y: clamp(at.y, document.documentElement.clientHeight),
    }
    openRef.current = centre
    hoveredRef.current = null
    setOpen(centre)
    setHovered(null)
    setCursor(null)
  }, [])

  const key = KEYMAPS[draft.keyboardLayout].wheel
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() !== key || event.repeat || openRef.current) return
      if (event.ctrlKey || event.metaKey || event.altKey || typing(event.target)) return
      const rect = board.current?.getBoundingClientRect()
      if (!rect) return
      show(pointer.current ?? { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 })
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [board, key, show])

  // Bound from the start: a listener added once the press is under way may not block the scroll.
  useEffect(() => {
    const element = board.current
    const onTouchMove = (event: TouchEvent) => {
      if (openRef.current) event.preventDefault()
    }
    element?.addEventListener('touchmove', onTouchMove, { passive: false })
    return () => element?.removeEventListener('touchmove', onTouchMove)
  }, [board])

  const { selectColour, selectShape, rotate } = draft
  const shapes = useMemo(() => freeShapes(draft.pieces), [draft.pieces])
  useEffect(() => {
    if (!open) return
    const counts = { colour: PIECE_COLOURS.length, shape: shapes.length }
    const onPointerMove = (event: PointerEvent) => {
      const [dx, dy] = [event.clientX - open.x, event.clientY - open.y]
      setCursor(clampToWheel(dx, dy))
      const slice = wheelSliceAt(dx, dy, counts)
      if (sameSlice(slice, hoveredRef.current)) return
      hoveredRef.current = slice
      setHovered(slice)
      if (slice?.ring === 'colour') selectColour(PIECE_COLOURS[slice.index])
    }
    const release = () => {
      const slice = hoveredRef.current
      if (slice?.ring === 'shape') selectShape(shapes[slice.index])
      show(null)
    }
    const onKeyUp = (event: KeyboardEvent) => {
      if (event.key.toLowerCase() === key) release()
    }
    const close = () => show(null)
    const onWheel = (event: WheelEvent) => {
      event.preventDefault()
      rotate()
    }
    // The system cursor would wander off the wheel: the wheel draws its own instead.
    document.documentElement.classList.add(...HIDE_CURSOR)
    window.addEventListener('pointermove', onPointerMove)
    window.addEventListener('keyup', onKeyUp)
    window.addEventListener('touchend', release)
    window.addEventListener('touchcancel', close)
    window.addEventListener('blur', close)
    window.addEventListener('wheel', onWheel, { passive: false })
    return () => {
      document.documentElement.classList.remove(...HIDE_CURSOR)
      window.removeEventListener('pointermove', onPointerMove)
      window.removeEventListener('keyup', onKeyUp)
      window.removeEventListener('touchend', release)
      window.removeEventListener('touchcancel', close)
      window.removeEventListener('blur', close)
      window.removeEventListener('wheel', onWheel)
    }
  }, [open, key, shapes, selectColour, selectShape, rotate, show])

  const cancelPress = () => window.clearTimeout(press.current?.timer)
  useEffect(
    () => () => {
      window.clearTimeout(press.current?.timer)
      dropGuard.current?.()
    },
    []
  )

  /** Spread on the board: remembers the mouse for the wheel key. */
  const onPointerMove = (event: React.PointerEvent) => {
    if (event.pointerType !== 'touch') pointer.current = { x: event.clientX, y: event.clientY }
  }

  /** Spread on the cells: a touch long press on an empty one opens the wheel there. */
  const cellHandlers = {
    onPointerDown: (event: React.PointerEvent) => {
      cancelPress()
      press.current = null
      const target = event.target as HTMLElement
      if (event.pointerType !== 'touch' || target.dataset.piece) return
      const at = { x: event.clientX, y: event.clientY }
      const timer = window.setTimeout(() => {
        dropGuard.current?.()
        dropGuard.current = swallowNextClick()
        show(at)
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

  return { open, hovered, cursor, onPointerMove, cellHandlers }
}
