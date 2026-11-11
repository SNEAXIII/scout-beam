import type { PieceColour } from './pieces'

export type BeamColour =
  'none' | 'white' | 'red' | 'blue' | 'yellow' | 'violet' | 'orange' | 'green' | 'black'

interface Swatch {
  readonly name: string
  readonly fill: string
  /** Text drawn on the fill. */
  readonly ink: string
  /** Set where the fill alone would vanish on the dark background. */
  readonly outline?: string
}

// Game colours are data, not theme: they stay fixed whatever the theme tokens say.
export const BEAM_SWATCHES: Record<BeamColour, Swatch> = {
  none: { name: 'Aucune', fill: 'transparent', ink: '#d4d4d8', outline: '#71717a' },
  white: { name: 'Blanc', fill: '#f4f4f5', ink: '#18181b' },
  red: { name: 'Rouge', fill: '#ef4444', ink: '#ffffff' },
  blue: { name: 'Bleu', fill: '#3b82f6', ink: '#ffffff' },
  yellow: { name: 'Jaune', fill: '#facc15', ink: '#18181b' },
  violet: { name: 'Violet', fill: '#a855f7', ink: '#ffffff' },
  orange: { name: 'Orange', fill: '#f97316', ink: '#18181b' },
  green: { name: 'Vert', fill: '#22c55e', ink: '#18181b' },
  black: { name: 'Noir', fill: '#000000', ink: '#fafafa', outline: '#d4d4d8' },
}

export const PIECE_SWATCHES: Record<PieceColour, Swatch> = {
  red: BEAM_SWATCHES.red,
  blue: BEAM_SWATCHES.blue,
  yellow: BEAM_SWATCHES.yellow,
  white: BEAM_SWATCHES.white,
}
