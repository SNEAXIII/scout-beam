import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import GameStatusBadge from '@/app/_components/game-status-badge'
import type { GameStatus } from '@/app/services/games'

interface GameHeaderProps {
  readonly status: GameStatus | null
}

export default function GameHeader({ status }: GameHeaderProps) {
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
    </header>
  )
}
