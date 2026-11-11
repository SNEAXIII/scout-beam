import { CheckCircle2, Flag, Hourglass } from 'lucide-react'
import { cn } from '@/app/lib/utils'
import type { GameStatus } from '@/app/services/games'

const BADGES = {
  in_progress: {
    Icon: Hourglass,
    label: 'En cours',
    tone: 'bg-secondary text-secondary-foreground',
  },
  solved: { Icon: CheckCircle2, label: 'Résolue', tone: 'bg-info text-info-foreground' },
  surrendered: { Icon: Flag, label: 'Abandonnée', tone: 'bg-muted text-muted-foreground' },
} as const

interface GameStatusBadgeProps {
  readonly status: GameStatus
}

export default function GameStatusBadge({ status }: GameStatusBadgeProps) {
  const { Icon, label, tone } = BADGES[status]
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium',
        tone
      )}
      data-status={status}
    >
      <Icon className='size-3' />
      {label}
    </span>
  )
}
