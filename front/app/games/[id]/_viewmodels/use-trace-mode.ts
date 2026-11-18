import { useCallback, useEffect, useState } from 'react'

/** The selected trace only, or every trace at once. */
export type TraceMode = 'selected' | 'all'

const STORAGE_KEY = 'scout-beam.trace-mode'

export function useTraceMode() {
  const [mode, setMode] = useState<TraceMode>('selected')

  useEffect(() => {
    try {
      if (localStorage.getItem(STORAGE_KEY) === 'all') setMode('all')
    } catch {
      // Storage unavailable: keep the default.
    }
  }, [])

  const choose = useCallback((next: TraceMode) => {
    setMode(next)
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // Storage unavailable: the choice lasts until the page is left.
    }
  }, [])

  return { mode, choose }
}
