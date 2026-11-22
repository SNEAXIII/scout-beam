export type KeyboardLayout = 'azerty' | 'qwerty'

export interface Keymap {
  readonly shapePrevious: string
  readonly shapeNext: string
  readonly colourPrevious: string
  readonly colourNext: string
  /** Held down: the piece wheel. */
  readonly wheel: string
}

/** Lower-case keys, at the same physical positions in both keyboard layouts. */
export const KEYMAPS: Record<KeyboardLayout, Keymap> = {
  azerty: { shapePrevious: 'q', shapeNext: 'd', colourPrevious: 'a', colourNext: 'e', wheel: 's' },
  qwerty: { shapePrevious: 'a', shapeNext: 'd', colourPrevious: 'q', colourNext: 'e', wheel: 's' },
}
