'use client'

import { useRouter } from 'next/navigation'
import { LogOut } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { clearTokens } from '@/app/lib/session'
import NewGameButton from './new-game-button'

export default function HomeHeader() {
  const router = useRouter()

  const signOut = () => {
    clearTokens()
    router.replace('/login')
  }

  return (
    <header className='flex flex-wrap items-center justify-between gap-3'>
      <h1 className='text-2xl font-semibold tracking-tight'>Mes parties</h1>
      <div className='flex items-start gap-2'>
        <NewGameButton />
        <Button
          variant='ghost'
          size='icon'
          onClick={signOut}
          aria-label='Se déconnecter'
          title='Se déconnecter'
          data-cy='sign-out'
        >
          <LogOut />
        </Button>
      </div>
    </header>
  )
}
