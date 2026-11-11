import { Check, RotateCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { PIECE_SWATCHES } from '@/app/lib/colours'
import { useI18n } from '@/app/i18n'
import { PIECE_COLOURS, SHAPES } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'
import ShapeIcon from './shape-icon'
import type { Draft } from '../_viewmodels/use-draft'

interface DraftToolbarProps {
  readonly draft: Draft
}

/** Picks the shape the next click places, its rotation and its colour. */
export default function DraftToolbar({ draft }: DraftToolbarProps) {
  const { t } = useI18n()
  const fill = PIECE_SWATCHES[draft.colour].fill
  return (
    <div className='flex flex-wrap items-center gap-x-4 gap-y-2'>
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
              className={cn('relative [&_svg]:size-6', picked && 'bg-accent ring-2 ring-ring')}
              disabled={placed}
              onClick={() => draft.selectShape(shape)}
              aria-pressed={picked}
              aria-label={`${t.shapes[shape]}${placed ? t.draft.placedSuffix : ''}`}
              title={t.shapes[shape]}
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
        title={t.draft.rotateHint}
        data-cy='draft-rotate'
        data-rotation={draft.rotation}
      >
        <RotateCw />
        {t.draft.rotate}
        <kbd className='rounded border border-border px-1 text-[10px] text-muted-foreground'>R</kbd>
      </Button>
      <div
        className='flex gap-1.5'
        role='group'
        aria-label={t.draft.colour}
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
            aria-label={t.colours[colour]}
            title={t.colours[colour]}
            data-cy={`draft-colour-${colour}`}
          />
        ))}
      </div>
    </div>
  )
}
