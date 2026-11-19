import { useCallback, useEffect, useMemo, useState } from 'react'
import { clearDraft } from '@/app/lib/draft'
import { ENTRIES } from '@/app/lib/entries'
import type { Piece, Shape } from '@/app/lib/pieces'
import { traceBeam } from '@/app/lib/trace'
import type { Shot } from '@/app/services/games'
import { pairKey, shotPaths } from '@/app/games/[id]/_components/shot-paths'
import type { KeyedTrace } from '@/app/games/[id]/_viewmodels/use-traces'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from '../_lib/script'

export const TUTORIAL_DRAFT = 'tutorial'
const STORAGE_KEY = 'scout-beam.tutorial'

function shotOf(entry: number): Shot {
  const trace = traceBeam(TUTORIAL_LAYOUT, entry)
  return { entry, exit_entry: trace?.exit ?? entry, colour: trace?.colour ?? 'none' }
}

/** A Proposition is correct when every entry answers as the hidden layout does: same exit, same colour. */
export function isCorrect(pieces: readonly Piece[]): boolean {
  return ENTRIES.every((entry) => {
    const proposed = traceBeam({ pieces: [...pieces] }, entry)
    const hidden = traceBeam(TUTORIAL_LAYOUT, entry)
    return proposed?.exit === hidden?.exit && proposed?.colour === hidden?.colour
  })
}

/** The Tutorial's progress, played in the browser on `TUTORIAL_LAYOUT`; past the last step, done. */
export function useTutorial() {
  const [step, setStep] = useState(0)
  const [shots, setShots] = useState<readonly Shot[]>([])
  const [traceShown, setTraceShown] = useState(false)
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const current = TUTORIAL_STEPS.at(step)
  const done = current === undefined
  // Shot steps come first, one shot each: the step's shot is fired once there is one per step so far.
  const fired = current?.kind === 'shot' ? shots.length > step : false

  // A reload starts the Tutorial over, so a draft left from before would be out of place.
  useEffect(() => clearDraft(TUTORIAL_DRAFT), [])

  const paths = useMemo(() => shotPaths(shots), [shots])

  const ghosts = useMemo(() => {
    const opened: Piece[] = []
    const eliminated = new Set<Shape>()
    TUTORIAL_STEPS.slice(0, shots.length).forEach((played) => {
      if (played.kind !== 'shot') return
      opened.push(...(played.candidates ?? []))
      for (const shape of played.eliminated ?? []) eliminated.add(shape)
    })
    return opened.map((piece) => ({ piece, eliminated: eliminated.has(piece.shape) }))
  }, [shots])

  const trace = useMemo((): KeyedTrace | undefined => {
    const shot = fired ? shots[step] : undefined
    const traced = shot && traceBeam(TUTORIAL_LAYOUT, shot.entry)
    return traced && { ...traced, key: pairKey(traced.entry, traced.exit) }
  }, [fired, shots, step])

  const fire = useCallback(
    (entry: number) => {
      if (current?.kind !== 'shot' || fired || entry !== current.entry) return
      setShots((previous) => [...previous, shotOf(entry)])
    },
    [current, fired]
  )

  const next = () => {
    setTraceShown(false)
    setStep((previous) => previous + 1)
  }

  const restart = () => {
    clearDraft(TUTORIAL_DRAFT)
    setStep(0)
    setShots([])
    setTraceShown(false)
  }

  const finish = () => {
    clearDraft(TUTORIAL_DRAFT)
    try {
      localStorage.setItem(STORAGE_KEY, 'done')
    } catch {
      // Blocked storage only means the home page offers the Tutorial again.
    }
    setStep(TUTORIAL_STEPS.length)
  }

  return {
    step,
    current,
    done,
    fired,
    shots,
    paths,
    ghosts,
    trace,
    traceShown,
    toggleTrace: () => setTraceShown((shown) => !shown),
    highlighted,
    setHighlighted,
    fire,
    next,
    restart,
    finish,
  }
}

export type Tutorial = ReturnType<typeof useTutorial>
