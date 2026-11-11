'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { createGame } from '@/app/services/games'

export default function NewGameButton() {
  const router = useRouter()
  const [pending, setPending] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const start = async () => {
    setPending(true)
    setError(null)
    try {
      const game = await createGame()
      router.push(`/games/${game.id}`)
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : 'Création impossible')
      setPending(false)
    }
  }

  return (
    <div className='flex flex-col items-end gap-2'>
      <Button
        onClick={() => void start()}
        disabled={pending}
        data-cy='new-game'
      >
        <Plus />
        Nouvelle partie
      </Button>
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='new-game-error'
        >
          {error}
        </p>
      )}
    </div>
  )
}
