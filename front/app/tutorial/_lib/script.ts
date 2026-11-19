import type { Layout, Piece, Shape } from '../../lib/pieces.ts'

/** One step of the Tutorial; a shot's answer comes from `traceBeam(TUTORIAL_LAYOUT, entry)`. */
export type TutorialStep =
  | {
      kind: 'shot'
      /** The only entry the player may fire. */
      entry: number
      /** Ghost pieces this shot opens, each giving the same answers as the shots fired so far. */
      candidates?: Piece[]
      /** Shapes of the candidates this shot rules out. */
      eliminated?: Shape[]
    }
  | { kind: 'draft' }
  | { kind: 'propose' }

const SQUARE: Piece = { shape: 'square', colour: 'red', x: 2, y: 3, rotation: 0 }
const CORNER: Piece = { shape: 'corner', colour: 'yellow', x: 4, y: 3, rotation: 2 }

export const TUTORIAL_LAYOUT: Layout = { pieces: [SQUARE, CORNER] }

export const TUTORIAL_STEPS: readonly TutorialStep[] = [
  { kind: 'shot', entry: 1 },
  { kind: 'shot', entry: 3 },
  {
    kind: 'shot',
    entry: 23,
    candidates: [
      { shape: 'spike', colour: 'yellow', x: 5, y: 4, rotation: 2 },
      CORNER,
      { shape: 'diamond', colour: 'yellow', x: 5, y: 3, rotation: 0 },
    ],
  },
  { kind: 'shot', entry: 7, eliminated: ['diamond'] },
  { kind: 'shot', entry: 5, eliminated: ['spike'] },
  { kind: 'shot', entry: 24 },
  { kind: 'draft' },
  { kind: 'propose' },
]
