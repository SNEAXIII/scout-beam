'use client'

import { useRouter } from 'next/navigation'
import { LogOut } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'
import { clearTokens } from '@/app/lib/session'
import NewGameButton from './new-game-button'

export default function HomeHeader() {
  const { t } = useI18n()
  const router = useRouter()

  const signOut = () => {
    clearTokens()
    router.replace('/login')
  }

  return (
    <header className='flex flex-wrap items-center justify-between gap-3'>
      <h1 className='text-2xl font-semibold tracking-tight'>{t.home.title}</h1>
      <div className='flex items-start gap-2'>
        <NewGameButton />
        <Button
          variant='ghost'
          size='icon'
          onClick={signOut}
          aria-label={t.home.signOut}
          title={t.home.signOut}
          data-cy='sign-out'
        >
          <LogOut />
        </Button>
      </div>
    </header>
  )
}
