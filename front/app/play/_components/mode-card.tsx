import Link from 'next/link'
import { ChevronRight, type LucideIcon } from 'lucide-react'
import { cn } from '@/app/lib/utils'

interface ModeCardProps {
  readonly icon: LucideIcon
  readonly title: string
  readonly text: string
  readonly cy: string
  readonly href?: string
  readonly onClick?: () => void
  readonly selected?: boolean
  readonly disabled?: boolean
}

/** One way to play: icon tile, title, two lines of text, a chevron; a link or a button. */
export default function ModeCard({
  icon: Icon,
  title,
  text,
  cy,
  href,
  onClick,
  selected,
  disabled,
}: ModeCardProps) {
  const className = cn(
    'flex w-full items-center gap-4 rounded-2xl border bg-card p-4 text-left transition-colors hover:bg-accent disabled:opacity-50',
    'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50',
    selected && 'border-2 border-primary bg-accent'
  )
  const body = (
    <>
      <span className='flex size-14 shrink-0 items-center justify-center rounded-xl bg-primary-soft text-primary'>
        <Icon className='size-7' />
      </span>
      <span className='flex min-w-0 flex-1 flex-col gap-1'>
        <span className='text-lg font-bold'>{title}</span>
        <span className='text-sm text-muted-foreground'>{text}</span>
      </span>
      <ChevronRight className='size-5 shrink-0' />
    </>
  )
  if (href)
    return (
      <Link
        href={href}
        className={className}
        data-cy={cy}
      >
        {body}
      </Link>
    )
  return (
    <button
      type='button'
      onClick={onClick}
      disabled={disabled}
      aria-expanded={selected}
      className={className}
      data-cy={cy}
    >
      {body}
    </button>
  )
}
