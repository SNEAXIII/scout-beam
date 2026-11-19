import { MousePointer2, Pointer } from 'lucide-react'
import { cn } from '@/app/lib/utils'

interface TapHintProps {
  readonly className?: string
  readonly style?: React.CSSProperties
}

/** A finger, or a mouse pointer on a fine pointer, pressing the centre of its parent in a loop. */
export default function TapHint({ className, style }: TapHintProps) {
  return (
    <span
      className={cn('pointer-events-none absolute inset-0 z-10', className)}
      style={style}
      aria-hidden
      data-cy='tap-hint'
    >
      <span className='absolute top-1/2 left-1/2 size-6 -translate-1/2 rounded-full border-2 border-primary opacity-0 motion-safe:animate-[tap-ripple_1.8s_ease-out_infinite]' />
      <span className='absolute top-1/2 left-1/2 origin-top-left motion-safe:animate-[tap-move_1.8s_ease-in-out_infinite]'>
        {/* Offsets put each icon's tip on the centre. */}
        <Pointer className='size-6 -translate-x-1/3 -translate-y-[8%] fill-background text-foreground drop-shadow-sm pointer-fine:hidden md:size-7' />
        <MousePointer2 className='hidden size-6 -translate-[18%] fill-background text-foreground drop-shadow-sm pointer-fine:block md:size-7' />
      </span>
    </span>
  )
}
