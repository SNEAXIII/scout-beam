import { BEAM_SWATCHES, BOUNCE_RING, MISS_MARKER } from '@/app/lib/colours'
import { cn } from '@/app/lib/utils'
import type { ShotPath } from './shot-paths'

interface BeamMarkerProps {
  readonly path: ShotPath
  readonly className?: string
}

/** A dot in the beam colour carrying the path's label, the same on both of its ends. */
export default function BeamMarker({ path, className }: BeamMarkerProps) {
  const swatch = BEAM_SWATCHES[path.colour]
  const isReturn = path.from === path.to
  const isMiss = path.colour === 'none'
  const bounced = !isReturn && !isMiss
  const style = (() => {
    if (isMiss)
      return {
        backgroundColor: MISS_MARKER.fill,
        color: swatch.ink,
        borderColor: MISS_MARKER.border,
      }
    if (bounced)
      return {
        // The fill stops at the padding so the dashes' gaps show the black layer beneath.
        background: `linear-gradient(${swatch.fill}, ${swatch.fill}) padding-box, ${BOUNCE_RING.gap} border-box`,
        color: swatch.ink,
        borderColor: BOUNCE_RING.dash,
      }
    return { backgroundColor: swatch.fill, color: swatch.ink, borderColor: swatch.outline }
  })()
  return (
    <span
      className={cn(
        'inline-flex size-5 shrink-0 items-center justify-center rounded-full text-[10px] leading-none font-bold',
        isReturn && swatch.outline && 'border',
        isMiss && 'border',
        bounced && 'border-2 border-dashed',
        className
      )}
      style={style}
      data-cy={`marker-${path.label}`}
      data-colour={path.colour}
      data-return={isReturn}
    >
      {path.label}
    </span>
  )
}
