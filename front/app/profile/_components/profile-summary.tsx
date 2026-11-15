'use client'

import Link from 'next/link'
import { UserPlus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { fill, useI18n } from '@/app/i18n'
import type { UserProfile } from '@/app/services/auth'

const DATE_STYLE: Intl.DateTimeFormatOptions = { dateStyle: 'long' }

export default function ProfileSummary({ profile }: Readonly<{ profile: UserProfile }>) {
  const { t, locale } = useI18n()
  const since = new Date(profile.created_at).toLocaleDateString(locale, DATE_STYLE)

  return (
    <Card data-cy='profile-summary'>
      <CardHeader>
        <CardTitle
          className='text-xl'
          data-cy='profile-login'
        >
          {profile.login}
        </CardTitle>
        <CardDescription>
          {profile.guest ? t.profile.guest : fill(t.profile.memberSince, { date: since })}
        </CardDescription>
      </CardHeader>
      {profile.guest && (
        <CardFooter className='flex flex-wrap items-center justify-between gap-3'>
          <p className='text-sm text-muted-foreground'>{t.profile.guestHint}</p>
          <Button
            asChild
            variant='outline'
          >
            <Link
              href='/register?next=/profile'
              data-cy='profile-register'
            >
              <UserPlus />
              {t.profile.guestCta}
            </Link>
          </Button>
        </CardFooter>
      )}
    </Card>
  )
}
