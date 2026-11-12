import Link from 'next/link'
import { ArrowLeft, Flag } from 'lucide-react'
import { Button } from '@/components/ui/button'
import GameStatusBadge from '@/app/_components/game-status-badge'
import { useI18n } from '@/app/i18n'
import type { GameStatus } from '@/app/services/games'

interface GameHeaderProps {
  readonly status: GameStatus | null
  /** Shown only while the game is in progress. */
  readonly onSurrender: () => void
  readonly surrendering: boolean
}

export default function GameHeader({ status, onSurrender, surrendering }: GameHeaderProps) {
  const { t } = useI18n()
  return (
    <header className='flex items-center gap-3'>
      <Button
        asChild
        variant='ghost'
        size='icon'
      >
        <Link
          href='/'
          aria-label={t.game.back}
          title={t.game.back}
          data-cy='back-to-games'
        >
          <ArrowLeft />
        </Link>
      </Button>
      <h1 className='text-2xl font-semibold tracking-tight'>{t.game.title}</h1>
      {status && <GameStatusBadge status={status} />}
      {status === 'in_progress' && (
        <Button
          variant='outline'
          className='ml-auto'
          onClick={onSurrender}
          disabled={surrendering}
          data-cy='surrender'
        >
          <Flag />
          {t.game.surrender}
        </Button>
      )}
    </header>
  )
}
