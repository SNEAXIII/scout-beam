import { useCallback, useMemo, useState } from 'react'
import { ENTRIES } from '@/app/lib/entries'
import type { Layout } from '@/app/lib/pieces'
import { traceBeam, type Trace } from '@/app/lib/trace'
import { pairKey } from '../_components/shot-paths'

export interface KeyedTrace extends Trace {
  /** `pairKey` of its two ends: the key of the shot path it draws, when one was fired. */
  readonly key: string
}

export interface Traces {
  /** One per pair of entries, none while the layout is hidden. */
  readonly traces: readonly KeyedTrace[]
  readonly keyAt: (entry: number) => string | undefined
  readonly selected: string | null
  /** The selected trace, running from the end the player picked. */
  readonly selectedTrace: KeyedTrace | undefined
  /** Selects the trace `entry` lies on, or clears the selection when it is already selected. */
  readonly toggle: (entry: number) => void
}

interface Selection {
  readonly key: string
  readonly from: number
}

function orient(trace: KeyedTrace, from: number): KeyedTrace {
  if (trace.entry === from) return trace
  return { ...trace, entry: trace.exit, exit: trace.entry, points: trace.points.toReversed() }
}

/** Both ends of a beam give the same trace: kept once, under the first end. */
function traceLayout(layout?: Layout) {
  const byKey = new Map<string, KeyedTrace>()
  const keys = new Map<number, string>()
  if (!layout) return { traces: [], keys }
  for (const entry of ENTRIES) {
    const trace = traceBeam(layout, entry)
    if (!trace) continue
    const key = pairKey(trace.entry, trace.exit)
    keys.set(entry, key)
    if (!byKey.has(key)) byKey.set(key, { ...trace, key })
  }
  return { traces: [...byKey.values()], keys }
}

/** The traces of the revealed layout, and the one the player picked to see. */
export function useTraces(layout?: Layout): Traces {
  const [selection, setSelection] = useState<Selection | null>(null)
  const { traces, keys } = useMemo(() => traceLayout(layout), [layout])

  const keyAt = useCallback((entry: number) => keys.get(entry), [keys])
  const toggle = useCallback(
    (entry: number) => {
      const key = keys.get(entry)
      if (key) setSelection((current) => (current?.key === key ? null : { key, from: entry }))
    },
    [keys]
  )
  const selectedTrace = useMemo(() => {
    const trace = selection && traces.find(({ key }) => key === selection.key)
    return trace ? orient(trace, selection.from) : undefined
  }, [traces, selection])
  return { traces, keyAt, selected: selection?.key ?? null, selectedTrace, toggle }
}
