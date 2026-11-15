'use client'

import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import InspirationSection from './_components/inspiration-section'
import LandingHero from './_components/landing-hero'
import RulesSection from './_components/rules-section'

export default function RulesPage() {
  const { t } = useI18n()
  return (
    <main className='relative mx-auto flex w-full max-w-3xl flex-col gap-12 px-4 py-16 md:px-8'>
      <Button
        asChild
        variant='ghost'
        size='icon'
        className='absolute top-2 left-4'
      >
        <Link
          href='/'
          aria-label={t.landing.back}
          title={t.landing.back}
          data-cy='rules-back'
        >
          <ArrowLeft />
        </Link>
      </Button>
      <div className='absolute top-2 right-4'>
        <LanguageSwitcher />
      </div>
      <LandingHero />
      <RulesSection />
      <InspirationSection />
    </main>
  )
}
