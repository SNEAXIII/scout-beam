'use client'

import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { CalendarDays, CircleHelp, LogOut, Trophy, UserRound } from 'lucide-react'
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
            href='/'
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
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/rules'
            aria-label={t.landing.rulesTitle}
            title={t.landing.rulesTitle}
            data-cy='rules-link'
          >
            <CircleHelp />
          </Link>
        </Button>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/profile'
            aria-label={t.play.profile}
            title={t.play.profile}
            data-cy='profile-link'
          >
            <UserRound />
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
