import { CheckCircle2, Hourglass } from 'lucide-react'
import { cn } from '@/app/lib/utils'
import type { GameStatus } from '@/app/services/games'

interface GameStatusBadgeProps {
  readonly status: GameStatus
}

export default function GameStatusBadge({ status }: GameStatusBadgeProps) {
  const solved = status === 'solved'
  const Icon = solved ? CheckCircle2 : Hourglass
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium',
        solved ? 'bg-info text-info-foreground' : 'bg-secondary text-secondary-foreground'
      )}
      data-status={status}
    >
      <Icon className='size-3' />
      {solved ? 'Résolue' : 'En cours'}
    </span>
  )
}
