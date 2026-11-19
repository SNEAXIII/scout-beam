'use client'

import Link from 'next/link'
import { GraduationCap, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'

export default function LandingHero() {
  const { t } = useI18n()
  return (
    <header className='flex flex-col items-center gap-4 text-center'>
      <h1 className='text-4xl font-semibold tracking-tight md:text-5xl'>{t.landing.title}</h1>
      <p className='max-w-md text-muted-foreground'>{t.landing.tagline}</p>
      <div className='flex flex-wrap items-center justify-center gap-3'>
        <Button
          asChild
          size='lg'
        >
          <Link
            href='/play'
            data-cy='play-cta'
          >
            <Play />
            {t.landing.play}
          </Link>
        </Button>
        <Button
          asChild
          variant='outline'
          size='lg'
        >
          <Link
            href='/tutorial'
            data-cy='rules-tutorial'
          >
            <GraduationCap />
            {t.tutorial.title}
          </Link>
        </Button>
        <Button
          asChild
          variant='ghost'
          size='lg'
        >
          <Link
            href='/login'
            data-cy='sign-in-link'
          >
            {t.landing.signIn}
          </Link>
        </Button>
      </div>
    </header>
  )
}
