import { useState } from 'react'
import Link from 'next/link'
import { ArrowLeft, Flag } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ConfirmationDialog } from '@/components/confirmation-dialog'
import LanguageSwitcher from '@/components/language-switcher'
import GameStatusBadge from '@/app/_components/game-status-badge'
import { useI18n } from '@/app/i18n'
import type { KeyboardLayout } from '@/app/lib/keyboard'
import type { GameStatus } from '@/app/services/games'
import KeyboardLayoutToggle from './keyboard-layout-toggle'

interface GameHeaderProps {
  readonly status: GameStatus | null
  /** Shown only while the game is in progress. */
  readonly onSurrender: () => void
  readonly surrendering: boolean
  readonly keyboardLayout: KeyboardLayout
  readonly onToggleKeyboardLayout: () => void
}

export default function GameHeader({
  status,
  onSurrender,
  surrendering,
  keyboardLayout,
  onToggleKeyboardLayout,
}: GameHeaderProps) {
  const { t } = useI18n()
  const [confirming, setConfirming] = useState(false)
  return (
    <header className='flex flex-wrap items-center gap-2 pt-2'>
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
      <h1 className='text-xl font-semibold tracking-tight'>{t.game.title}</h1>
      {status && <GameStatusBadge status={status} />}
      <div className='ml-auto flex items-center gap-2'>
        {status === 'in_progress' && (
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
