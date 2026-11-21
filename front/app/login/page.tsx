'use client'

import { Suspense, useEffect, useState } from 'react'
import { listProviders, type Providers } from '@/app/services/auth'
import DevLoginForm from './_components/dev-login-form'
import LoginForm from './_components/login-form'

const NONE: Providers = { dev_login: false, providers: [] }

export default function LoginPage() {
  const [offer, setOffer] = useState<Providers>(NONE)

  useEffect(() => {
    // An unreachable API offers nothing extra; the password form reports the failure itself.
    listProviders().then(setOffer, () => setOffer(NONE))
  }, [])

  return (
    <main className='flex flex-1 items-center justify-center p-4'>
      <div className='flex w-full max-w-sm flex-col gap-4'>
        <Suspense>
          <LoginForm providers={offer.providers} />
          {offer.dev_login && <DevLoginForm />}
        </Suspense>
      </div>
    </main>
  )
}
