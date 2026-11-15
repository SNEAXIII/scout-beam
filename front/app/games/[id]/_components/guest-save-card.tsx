'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { UserPlus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { getSession } from '@/app/services/auth'

/** For a guest who solved a daily: signing up converts the guest, so the game is kept. */
export default function GuestSaveCard() {
  const { t } = useI18n()
  const [guest, setGuest] = useState(false)

  useEffect(() => {
    getSession().then(
      (profile) => setGuest(profile.guest),
      () => setGuest(false)
    )
  }, [])

  if (!guest) return null
  return (
    <Card data-cy='guest-save-card'>
      <CardHeader>
        <CardTitle>{t.daily.saveTitle}</CardTitle>
        <CardDescription>{t.daily.saveText}</CardDescription>
      </CardHeader>
      <CardContent>
        <Button asChild>
          <Link
            href='/register'
            data-cy='guest-save-register'
          >
            <UserPlus />
            {t.daily.saveCta}
          </Link>
        </Button>
      </CardContent>
    </Card>
  )
}
