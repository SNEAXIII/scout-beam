'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'

export default function HomePage() {
  const { t } = useI18n()
  return (
    <main className='flex min-h-screen items-center justify-center p-4'>
      <Button asChild>
        <Link
          href='/play'
          data-cy='play-cta'
        >
          {t.play.title}
        </Link>
      </Button>
    </main>
  )
}
