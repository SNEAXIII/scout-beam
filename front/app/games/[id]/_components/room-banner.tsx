import { Flag, Trophy, XCircle } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import type { GameStatus } from '@/app/services/games'

const BANNERS = {
  solved: { Icon: Trophy, key: 'won', tone: 'bg-info text-info-foreground' },
  lost: { Icon: XCircle, key: 'lost', tone: 'bg-muted text-foreground' },
  surrendered: { Icon: Flag, key: 'surrendered', tone: 'bg-muted text-foreground' },
} as const

/** A room's end: whoever proposed right first won; a surrender hands the win to the opponent. */
export default function RoomBanner({
  status,
}: Readonly<{ status: Exclude<GameStatus, 'in_progress'> }>) {
  const { t } = useI18n()
  const { Icon, key, tone } = BANNERS[status]
  return (
    <p
      className={cn('flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium', tone)}
      role='status'
      data-cy='room-banner'
      data-status={key}
    >
      <Icon className='size-4 shrink-0' />
      {t.rooms[key]}
    </p>
  )
}
