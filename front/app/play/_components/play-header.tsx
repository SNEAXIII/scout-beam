'use client'

import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { CalendarDays, LogOut, Trophy } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import { logout } from '@/app/services/auth'
import LanguageSwitcher from '@/components/language-switcher'
import NewGameButton from './new-game-button'

export default function PlayHeader() {
  const { t } = useI18n()
  const router = useRouter()

  // Off to /login even when the API is unreachable: its cookies then expire on their own.
  const signOut = () => {
    void logout()
      .catch(() => undefined)
      .finally(() => router.replace('/login'))
  }

  return (
    <header className='flex flex-wrap items-center justify-between gap-3'>
      <h1 className='text-2xl font-semibold tracking-tight'>{t.play.title}</h1>
      <div className='flex items-start gap-2'>
        <Button
          asChild
          variant='outline'
        >
          <Link
            href='/daily'
            data-cy='daily-link'
          >
            <CalendarDays />
            {t.play.daily}
          </Link>
        </Button>
        <NewGameButton />
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/ranking'
            aria-label={t.play.ranking}
            title={t.play.ranking}
            data-cy='ranking-link'
          >
            <Trophy />
          </Link>
        </Button>
        <Button
          variant='ghost'
          size='icon'
          onClick={signOut}
          aria-label={t.play.signOut}
          title={t.play.signOut}
          data-cy='sign-out'
        >
          <LogOut />
        </Button>
        <LanguageSwitcher />
      </div>
    </header>
  )
}
