'use client'

import InspirationSection from './_components/inspiration-section'
import LandingHero from './_components/landing-hero'
import RulesSection from './_components/rules-section'

export default function RulesPage() {
  return (
    <main className='mx-auto flex w-full max-w-3xl flex-col gap-12 px-4 py-16 md:px-8'>
      <LandingHero />
      <RulesSection />
      <InspirationSection />
    </main>
  )
}
