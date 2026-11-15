import { useState } from 'react'
import Link from 'next/link'
import { ArrowLeft, Flag } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ConfirmationDialog } from '@/components/confirmation-dialog'
import LanguageSwitcher from '@/components/language-switcher'
import GameStatusBadge from '@/app/play/_components/game-status-badge'
import { useI18n } from '@/app/i18n'
import type { KeyboardLayout } from '@/app/lib/keyboard'
import type { GameView } from '@/app/services/games'
import DailyTimer from './daily-timer'
import KeyboardLayoutToggle from './keyboard-layout-toggle'

interface GameHeaderProps {
  readonly game: GameView | null
  /** Shown only while the game is in progress, and never on a daily. */
  readonly onSurrender: () => void
  readonly surrendering: boolean
  readonly keyboardLayout: KeyboardLayout
  readonly onToggleKeyboardLayout: () => void
}

export default function GameHeader({
  game,
  onSurrender,
  surrendering,
  keyboardLayout,
  onToggleKeyboardLayout,
}: GameHeaderProps) {
  const { t } = useI18n()
  const [confirming, setConfirming] = useState(false)
  const status = game?.status ?? null
  const daily = game?.daily_date != null
  return (
    <header className='flex flex-wrap items-center gap-2 pt-2'>
      <Button
        asChild
        variant='ghost'
        size='icon'
      >
        <Link
          href={daily ? '/daily' : '/play'}
          aria-label={t.game.back}
          title={t.game.back}
          data-cy='back-to-games'
        >
          <ArrowLeft />
        </Link>
      </Button>
      <h1 className='text-xl font-semibold tracking-tight'>{t.game.title}</h1>
      {status && <GameStatusBadge status={status} />}
      {game && daily && (
        <DailyTimer
          createdAt={game.created_at}
          solvedAt={game.solved_at}
          wrongPropositions={game.wrong_propositions}
        />
      )}
      <div className='ml-auto flex items-center gap-2'>
        {status === 'in_progress' && !daily && (
          <Button
            variant='outline'
            size='sm'
            onClick={() => setConfirming(true)}
            disabled={surrendering}
            data-cy='surrender'
          >
            <Flag />
            {t.game.surrender}
          </Button>
        )}
        {status === 'in_progress' && (
          <KeyboardLayoutToggle
            keyboardLayout={keyboardLayout}
            onToggle={onToggleKeyboardLayout}
          />
        )}
        <LanguageSwitcher />
      </div>
      <ConfirmationDialog
        open={confirming}
        onOpenChange={setConfirming}
        title={t.game.surrenderTitle}
        description={t.game.surrenderDescription}
        confirmText={t.game.surrender}
        variant='destructive'
        onConfirm={onSurrender}
        dataCy='surrender-dialog'
      />
    </header>
  )
}
