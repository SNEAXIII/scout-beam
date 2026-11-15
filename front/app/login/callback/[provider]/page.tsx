'use client'

import { useEffect, useRef, useState } from 'react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { isProviderName, oauthLogin, takeOAuthState } from '@/app/services/auth'

/** Where the provider sends the browser back: `state` is checked before anything is sent. */
export default function OAuthCallbackPage() {
  const { t } = useI18n()
  const router = useRouter()
  const { provider } = useParams<{ provider: string }>()
  const [error, setError] = useState<string | null>(null)
  // The stored state is read once: a second effect run (Strict Mode) would find it gone.
  const started = useRef(false)

  useEffect(() => {
    if (started.current) return
    started.current = true

    const params = new URLSearchParams(window.location.search)
    const code = params.get('code')
    const expected = takeOAuthState()
    if (!isProviderName(provider) || !code || !expected || params.get('state') !== expected) {
      setError(t.callback.invalid)
      return
    }

    oauthLogin(provider, code).then(
      () => router.replace('/'),
      (caught: unknown) => setError(caught instanceof Error ? caught.message : t.callback.failed)
    )
  }, [provider, router, t])

  return (
    <main className='flex flex-1 items-center justify-center p-4'>
      <Card className='w-full max-w-sm'>
        <CardHeader>
          <CardTitle>Scout Beam</CardTitle>
        </CardHeader>
        <CardContent className='flex flex-col gap-4'>
          {error ? (
            <>
              <p
                className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
                role='alert'
                data-cy='callback-error'
              >
                {error}
              </p>
              <Link
                href='/login'
                className='text-center text-sm text-primary underline-offset-4 hover:underline'
                data-cy='callback-back'
              >
                {t.callback.back}
              </Link>
            </>
          ) : (
            <p
              className='text-sm text-muted-foreground'
              data-cy='callback-pending'
            >
              {t.callback.pending}
            </p>
          )}
        </CardContent>
      </Card>
    </main>
  )
}
