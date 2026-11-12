import { Check, RotateCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { fill, useI18n } from '@/app/i18n'
import { KEYMAPS, type KeyboardLayout } from '@/app/lib/keyboard'
import { PIECE_COLOURS, SHAPES } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'
import ShapeIcon from './shape-icon'
import type { Draft } from '../_viewmodels/use-draft'

interface DraftToolbarProps {
  readonly draft: Draft
  readonly keyboardLayout: KeyboardLayout
}

/** Picks the shape the next click places, its rotation and its colour. */
export default function DraftToolbar({ draft, keyboardLayout }: DraftToolbarProps) {
  const { t } = useI18n()
  const swatch = PIECE_SWATCHES[draft.colour]
  const keys = KEYMAPS[keyboardLayout]
  const shapeKeys = {
    previous: keys.shapePrevious.toUpperCase(),
    next: keys.shapeNext.toUpperCase(),
  }
  const colourKeys = {
    previous: keys.colourPrevious.toUpperCase(),
    next: keys.colourNext.toUpperCase(),
  }
  return (
    <div className='flex flex-wrap items-center gap-x-3 gap-y-1.5'>
      <div
        className='flex gap-1'
        role='group'
        aria-label={t.draft.pieceToPlace}
      >
        {SHAPES.map((shape) => {
          const placed = draft.pieces.some((piece) => piece.shape === shape)
          const picked = draft.shape === shape
          return (
            <Button
              key={shape}
              variant='outline'
              size='icon'
              className={cn('relative [&_svg]:size-5', picked && 'bg-accent ring-2 ring-ring')}
              disabled={placed}
              onClick={() => draft.selectShape(shape)}
              aria-pressed={picked}
              aria-label={`${t.shapes[shape]}${placed ? t.draft.placedSuffix : ''}`}
              title={fill(t.draft.shapeHint, { shape: t.shapes[shape], ...shapeKeys })}
              data-cy={`draft-shape-${shape}`}
              data-placed={placed}
            >
              <ShapeIcon
                shape={shape}
                rotation={picked ? draft.rotation : 0}
                fill={picked ? swatch.fill : 'currentColor'}
              />
              {placed && (
                <Check className='absolute -top-1 -right-1 size-3.5! rounded-full bg-card' />
              )}
            </Button>
          )
        })}
      </div>
      <Button
        variant='outline'
        size='sm'
        onClick={draft.rotate}
        disabled={draft.shape === null}
        title={t.draft.rotateHint}
        data-cy='draft-rotate'
        data-rotation={draft.rotation}
      >
        <RotateCw />
        {t.draft.rotate}
        <kbd className='rounded border border-border px-1 text-[10px] text-muted-foreground'>R</kbd>
      </Button>
      <div
        className='flex gap-1'
        role='group'
        aria-label={t.draft.colour}
      >
        {PIECE_COLOURS.map((colour) => (
          <Button
            key={colour}
            variant='outline'
            size='icon'
            className={cn(
              'size-7 rounded-full transition-transform',
              draft.colour === colour
                ? 'ring-2 ring-ring ring-offset-1 ring-offset-background'
                : 'hover:scale-110'
            )}
            style={{ backgroundColor: PIECE_SWATCHES[colour].fill }}
            onClick={() => draft.selectColour(colour)}
            aria-pressed={draft.colour === colour}
            aria-label={t.colours[colour]}
            title={fill(t.draft.colourHint, { colour: t.colours[colour], ...colourKeys })}
            data-cy={`draft-colour-${colour}`}
          />
        ))}
      </div>
    </div>
  )
}
