import LanguageSwitcher from '@/components/language-switcher'
import InspirationSection from './_components/inspiration-section'
import LandingHero from './_components/landing-hero'
import RulesSection from './_components/rules-section'

export default function HomePage() {
  return (
    <main className='relative mx-auto flex w-full max-w-3xl flex-col gap-12 px-4 py-16 md:px-8'>
      <div className='absolute top-2 right-4'>
        <LanguageSwitcher />
      </div>
      <LandingHero />
      <RulesSection />
      <InspirationSection />
    </main>
  )
}
