import { Check, RotateCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { PIECE_COLOURS, SHAPE_NAMES, SHAPES } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'
import ShapeIcon from './shape-icon'
import type { Draft } from './use-draft'

interface DraftToolbarProps {
  readonly draft: Draft
}

/** Picks the shape the next click places, its rotation and its colour. */
export default function DraftToolbar({ draft }: DraftToolbarProps) {
  const fill = PIECE_SWATCHES[draft.colour].fill
  return (
    <div className='flex flex-wrap items-center gap-x-4 gap-y-2'>
      <div
        className='flex gap-1'
        role='group'
        aria-label='Pièce à placer'
      >
        {SHAPES.map((shape) => {
          const placed = draft.pieces.some((piece) => piece.shape === shape)
          const picked = draft.shape === shape
          return (
            <Button
              key={shape}
              variant='outline'
              size='icon'
              className={cn('relative [&_svg]:size-6', picked && 'bg-accent ring-2 ring-ring')}
              disabled={placed}
              onClick={() => draft.selectShape(shape)}
              aria-pressed={picked}
              aria-label={`${SHAPE_NAMES[shape]}${placed ? ' (placée)' : ''}`}
              title={SHAPE_NAMES[shape]}
              data-cy={`draft-shape-${shape}`}
              data-placed={placed}
            >
              <ShapeIcon
                shape={shape}
                rotation={picked ? draft.rotation : 0}
                fill={picked ? fill : 'currentColor'}
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
        title='Tourner d’un quart de tour (R)'
        data-cy='draft-rotate'
        data-rotation={draft.rotation}
      >
        <RotateCw />
        Tourner
        <kbd className='rounded border border-border px-1 text-[10px] text-muted-foreground'>R</kbd>
      </Button>
      <div
        className='flex gap-1.5'
        role='group'
        aria-label='Couleur'
      >
        {PIECE_COLOURS.map((colour) => (
          <Button
            key={colour}
            variant='outline'
            size='icon'
            className={cn(
              'rounded-full transition-transform',
              draft.colour === colour
                ? 'ring-2 ring-ring ring-offset-2 ring-offset-card'
                : 'hover:scale-110'
            )}
            style={{ backgroundColor: PIECE_SWATCHES[colour].fill }}
            onClick={() => draft.selectColour(colour)}
            aria-pressed={draft.colour === colour}
            aria-label={PIECE_SWATCHES[colour].name}
            title={PIECE_SWATCHES[colour].name}
            data-cy={`draft-colour-${colour}`}
          />
        ))}
      </div>
    </div>
  )
}
