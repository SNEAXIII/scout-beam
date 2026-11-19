import type { Translations } from '../../i18n/locales/en.ts'
import type { Layout, Piece } from '../../lib/pieces.ts'

type StepText = keyof Translations['tutorial']['steps']

/** One step of the Tutorial; a shot's answer comes from `traceBeam(TUTORIAL_LAYOUT, entry)`. */
export type TutorialStep = { text: StepText } & (
  | { kind: 'intro' }
  | {
      kind: 'shot'
      /** The only entry the player may fire. */
      entry: number
      /** Ghost pieces this shot opens, each giving the same answers as the shots fired so far. */
      candidates?: Piece[]
      /** Candidates this shot rules out, the very objects listed in `candidates`. */
      eliminated?: Piece[]
    }
  | { kind: 'draft' }
  | { kind: 'propose' }
)

const SQUARE: Piece = { shape: 'square', colour: 'red', x: 2, y: 3, rotation: 0 }
const CORNER: Piece = { shape: 'corner', colour: 'yellow', x: 4, y: 3, rotation: 2 }

const SPIKE: Piece = { shape: 'spike', colour: 'yellow', x: 5, y: 4, rotation: 2 }
const LOW_CORNER: Piece = { ...CORNER, x: 5, y: 4 }
const DIAMOND: Piece = { shape: 'diamond', colour: 'yellow', x: 5, y: 3, rotation: 0 }

export const TUTORIAL_LAYOUT: Layout = { pieces: [SQUARE, CORNER] }

/** The five shapes, apart, a row of names under each row of pieces. */
export const INTRO_LAYOUT: Layout = {
  pieces: [
    { shape: 'square', colour: 'red', x: 1, y: 2, rotation: 0 },
    { shape: 'bar', colour: 'blue', x: 4, y: 2, rotation: 0 },
    { shape: 'diamond', colour: 'yellow', x: 7, y: 1, rotation: 0 },
    { shape: 'spike', colour: 'white', x: 2, y: 5, rotation: 0 },
    { shape: 'corner', colour: 'blue', x: 6, y: 4, rotation: 0 },
  ],
}

export const TUTORIAL_STEPS: readonly TutorialStep[] = [
  { kind: 'intro', text: 'intro' },
  { kind: 'shot', text: 'step1', entry: 1 },
  { kind: 'shot', text: 'step2', entry: 3 },
  {
    kind: 'shot',
    text: 'step3',
    entry: 23,
    candidates: [SPIKE, CORNER, LOW_CORNER, DIAMOND],
  },
  { kind: 'shot', text: 'step4', entry: 7, eliminated: [DIAMOND, LOW_CORNER] },
  { kind: 'shot', text: 'step5', entry: 5, eliminated: [SPIKE] },
  { kind: 'shot', text: 'step6', entry: 24 },
  { kind: 'draft', text: 'step7' },
  { kind: 'propose', text: 'step8' },
]
