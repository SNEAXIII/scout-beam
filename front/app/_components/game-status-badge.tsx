import { CheckCircle2, Flag, Hourglass } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import type { GameStatus } from '@/app/services/games'

const BADGES = {
  in_progress: { Icon: Hourglass, tone: 'bg-secondary text-secondary-foreground' },
  solved: { Icon: CheckCircle2, tone: 'bg-info text-info-foreground' },
  surrendered: { Icon: Flag, tone: 'bg-muted text-muted-foreground' },
} as const

interface GameStatusBadgeProps {
  readonly status: GameStatus
}

export default function GameStatusBadge({ status }: GameStatusBadgeProps) {
  const { t } = useI18n()
  const { Icon, tone } = BADGES[status]
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium',
        tone
      )}
      data-status={status}
    >
      <Icon className='size-3' />
      {t.status[status]}
    </span>
  )
}
