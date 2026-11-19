import { useCallback, useEffect, useMemo, useState } from 'react'
import { clearDraft } from '@/app/lib/draft'
import { ENTRIES } from '@/app/lib/entries'
import type { Piece } from '@/app/lib/pieces'
import { traceBeam } from '@/app/lib/trace'
import type { Shot } from '@/app/services/games'
import { pairKey, shotPaths } from '@/app/games/[id]/_components/shot-paths'
import type { KeyedTrace } from '@/app/games/[id]/_viewmodels/use-traces'
import { TUTORIAL_LAYOUT, TUTORIAL_STEPS } from '../_lib/script'

const SHOT_STEPS = TUTORIAL_STEPS.filter((step) => step.kind === 'shot')

export const TUTORIAL_DRAFT = 'tutorial'
export const TUTORIAL_KEY = 'scout-beam.tutorial'

function shotOf(entry: number): Shot {
  const trace = traceBeam(TUTORIAL_LAYOUT, entry)
  return { entry, exit_entry: trace?.exit ?? entry, colour: trace?.colour ?? 'none' }
}

/** The Traces of the shots before `step`, kept drawn once explained. */
export const pastTraces = (step: number) =>
  TUTORIAL_STEPS.slice(0, step).flatMap((played) => {
    const traced = played.kind === 'shot' ? traceBeam(TUTORIAL_LAYOUT, played.entry) : undefined
    return traced ? [{ ...traced, key: pairKey(traced.entry, traced.exit) }] : []
  })

/** A Proposition is correct when every entry answers as the hidden layout does: same exit, same colour. */
export function isCorrect(pieces: readonly Piece[]): boolean {
  return ENTRIES.every((entry) => {
    const proposed = traceBeam({ pieces: [...pieces] }, entry)
    const hidden = traceBeam(TUTORIAL_LAYOUT, entry)
    return proposed?.exit === hidden?.exit && proposed?.colour === hidden?.colour
  })
}

/** Where a shot step stands: before its shot, its bare answer, then its Trace and explanation. */
export type Phase = 'aim' | 'answer' | 'explain'

/** The Tutorial's progress, played in the browser on `TUTORIAL_LAYOUT`; past the last step, done. */
export function useTutorial() {
  const [step, setStep] = useState(0)
  const [phase, setPhase] = useState<Phase>('aim')
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const current = TUTORIAL_STEPS.at(step)
  const done = current === undefined
  const fired = current?.kind === 'shot' && phase !== 'aim'

  // A reload starts the Tutorial over, so a draft left from before would be out of place.
  useEffect(() => clearDraft(TUTORIAL_DRAFT), [])

  // Derived from the position, so stepping back removes the shot with its markers and ghosts.
  const shots = useMemo(
    () =>
      TUTORIAL_STEPS.slice(0, fired ? step + 1 : step).flatMap((played) =>
        played.kind === 'shot' ? [shotOf(played.entry)] : []
      ),
    [step, fired]
  )

  const paths = useMemo(() => shotPaths(shots), [shots])

  const ghosts = useMemo(() => {
    const opened: Piece[] = []
    const eliminated = new Set<Piece>()
    SHOT_STEPS.slice(0, shots.length).forEach((played) => {
      opened.push(...(played.candidates ?? []))
      for (const piece of played.eliminated ?? []) eliminated.add(piece)
    })
    return opened.map((piece) => ({ piece, eliminated: eliminated.has(piece) }))
  }, [shots])

  const shot = fired ? shots.at(-1) : undefined

  const trace = useMemo((): KeyedTrace | undefined => {
    const traced = shot && traceBeam(TUTORIAL_LAYOUT, shot.entry)
    return traced && { ...traced, key: pairKey(traced.entry, traced.exit) }
  }, [shot])

  const fire = useCallback(
    (entry: number) => {
      if (current?.kind === 'shot' && !fired && entry === current.entry) setPhase('answer')
    },
    [current, fired]
  )

  const next = () => {
    setHighlighted(null)
    if (current?.kind === 'shot' && phase === 'answer') return setPhase('explain')
    setStep(step + 1)
    setPhase('aim')
  }

  const previous = () => {
    if (step === 0) return
    setHighlighted(null)
    if (current?.kind === 'shot' && phase !== 'aim') {
      return setPhase(phase === 'explain' ? 'answer' : 'aim')
    }
    setStep(step - 1)
    setPhase('explain')
  }

  const restart = () => {
    clearDraft(TUTORIAL_DRAFT)
    setHighlighted(null)
    setStep(0)
    setPhase('aim')
  }

  const finish = () => {
    clearDraft(TUTORIAL_DRAFT)
    try {
      localStorage.setItem(TUTORIAL_KEY, 'done')
    } catch {
      // Blocked storage only means the home page offers the Tutorial again.
    }
    setStep(TUTORIAL_STEPS.length)
  }

  return {
    step,
    phase,
    current,
    done,
    fired,
    shot,
    paths,
    ghosts,
    trace,
    highlighted,
    setHighlighted,
    fire,
    next,
    previous,
    restart,
    finish,
  }
}

export type Tutorial = ReturnType<typeof useTutorial>
