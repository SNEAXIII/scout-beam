import { BEAM_SWATCHES } from '@/app/lib/colours'
import { cn } from '@/app/lib/utils'
import type { ShotPath } from './shot-paths'

interface BeamMarkerProps {
  readonly path: ShotPath
  readonly className?: string
}

/** A dot in the beam colour carrying the path's label, the same on both of its ends. */
export default function BeamMarker({ path, className }: BeamMarkerProps) {
  const swatch = BEAM_SWATCHES[path.colour]
  return (
    <span
      className={cn(
        'inline-flex size-5 shrink-0 items-center justify-center rounded-full text-[10px] leading-none font-bold',
        swatch.outline && 'border',
        path.colour === 'none' && 'border-dashed',
        className
      )}
      style={{ backgroundColor: swatch.fill, color: swatch.ink, borderColor: swatch.outline }}
      data-cy={`marker-${path.label}`}
      data-colour={path.colour}
    >
      {path.label}
    </span>
  )
}
