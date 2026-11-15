'use client'

import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import LanguageSwitcher from '@/components/language-switcher'
import { useI18n } from '@/app/i18n'
import ProfileSummary from './_components/profile-summary'
import RenameForm from './_components/rename-form'
import { useProfile } from './_viewmodels/use-profile'

export default function ProfilePage() {
  const { t } = useI18n()
  const profile = useProfile()

  return (
    <main className='mx-auto flex w-full max-w-xl flex-col gap-6 p-4 md:p-8'>
      <header className='flex flex-wrap items-center gap-2'>
        <Button
          asChild
          variant='ghost'
          size='icon'
        >
          <Link
            href='/play'
            aria-label={t.profile.back}
            title={t.profile.back}
            data-cy='profile-back'
          >
            <ArrowLeft />
          </Link>
        </Button>
        <h1 className='text-2xl font-semibold tracking-tight'>{t.profile.title}</h1>
        <div className='ml-auto'>
          <LanguageSwitcher />
        </div>
      </header>
      {profile.profile ? (
        <>
          <ProfileSummary profile={profile.profile} />
          <RenameForm profile={profile} />
        </>
      ) : (
        <p
          className={
            profile.error
              ? 'text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              : 'text-sm text-muted-foreground'
          }
          role={profile.error ? 'alert' : undefined}
          data-cy='profile-loading'
        >
          {profile.error ?? t.common.loading}
        </p>
      )}
    </main>
  )
}
