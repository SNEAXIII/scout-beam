import { createPortal } from 'react-dom'
import { useI18n } from '@/app/i18n'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { PIECE_COLOURS } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'
import {
  sliceCentre,
  slicePath,
  WHEEL_DEAD_ZONE,
  WHEEL_RADIUS,
  type WheelRing,
  type WheelSlice,
} from '@/app/lib/wheel'
import ShapeIcon from '@/components/shape-icon'
import type { Draft } from '../_viewmodels/use-draft'
import { freeShapes } from '../_viewmodels/use-piece-wheel'

interface PieceWheelProps {
  readonly draft: Draft
  /** Viewport px, already clamped so the wheel fits. */
  readonly centre: { readonly x: number; readonly y: number }
  readonly hovered: WheelSlice | null
}

const SIZE = 2 * WHEEL_RADIUS

/** The radial picker, drawn over the page at `centre`; it never takes the pointer. */
export default function PieceWheel({ draft, centre, hovered }: PieceWheelProps) {
  const { t } = useI18n()
  const isHovered = (ring: WheelRing, index: number) =>
    hovered?.ring === ring && hovered.index === index
  const shapes = freeShapes(draft.pieces)
  let label = t.draft.wheel
  if (hovered?.ring === 'colour') label = t.colours[PIECE_COLOURS[hovered.index]]
  if (hovered?.ring === 'shape') label = t.shapes[shapes[hovered.index]]

  return createPortal(
    <div
      className='pointer-events-none fixed z-50'
      style={{
        left: centre.x - WHEEL_RADIUS,
        top: centre.y - WHEEL_RADIUS,
        width: SIZE,
        height: SIZE,
      }}
      data-cy='piece-wheel'
    >
      <svg
        viewBox={`${-WHEEL_RADIUS} ${-WHEEL_RADIUS} ${SIZE} ${SIZE}`}
        className='size-full overflow-visible drop-shadow-lg'
        aria-hidden
      >
        <circle
          r={WHEEL_DEAD_ZONE * WHEEL_RADIUS}
          className='fill-card stroke-border'
        />
        {PIECE_COLOURS.map((colour, index) => (
          <path
            key={colour}
            d={slicePath('colour', index, PIECE_COLOURS.length)}
            fill={PIECE_SWATCHES[colour].fill}
            className={colour === draft.colour ? 'stroke-foreground' : 'stroke-border'}
            strokeWidth={colour === draft.colour ? 3 : 1}
            data-cy={`wheel-colour-${colour}`}
            data-hovered={isHovered('colour', index)}
          />
        ))}
        {shapes.map((shape, index) => (
          <path
            key={shape}
            d={slicePath('shape', index, shapes.length)}
            className={cn('stroke-border', isHovered('shape', index) ? 'fill-accent' : 'fill-card')}
            data-cy={`wheel-shape-${shape}`}
            data-hovered={isHovered('shape', index)}
          />
        ))}
      </svg>
      {shapes.map((shape, index) => {
        const { x, y } = sliceCentre('shape', index, shapes.length)
        return (
          <div
            key={shape}
            className='absolute -translate-1/2'
            style={{ left: WHEEL_RADIUS + x, top: WHEEL_RADIUS + y }}
          >
            <ShapeIcon
              shape={shape}
              rotation={draft.rotation}
              fill={PIECE_SWATCHES[draft.colour].fill}
              className='size-7'
            />
          </div>
        )
      })}
      <span className='absolute top-1/2 left-1/2 w-14 -translate-1/2 text-center text-[11px] leading-tight font-medium text-foreground'>
        {label}
      </span>
    </div>,
    document.body
  )
}
