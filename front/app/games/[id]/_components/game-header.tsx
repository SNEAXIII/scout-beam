import Link from 'next/link'
import { ArrowLeft, Flag } from 'lucide-react'
import { Button } from '@/components/ui/button'
import GameStatusBadge from '@/app/_components/game-status-badge'
import type { GameStatus } from '@/app/services/games'

interface GameHeaderProps {
  readonly status: GameStatus | null
  /** Shown only while the game is in progress. */
  readonly onSurrender: () => void
  readonly surrendering: boolean
}

export default function GameHeader({ status, onSurrender, surrendering }: GameHeaderProps) {
  return (
    <header className='flex items-center gap-3'>
      <Button
        asChild
        variant='ghost'
        size='icon'
      >
        <Link
          href='/'
          aria-label='Retour aux parties'
          title='Retour aux parties'
          data-cy='back-to-games'
        >
          <ArrowLeft />
        </Link>
      </Button>
      <h1 className='text-2xl font-semibold tracking-tight'>Partie</h1>
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
          Se rendre
        </Button>
      )}
    </header>
  )
}
