'use client'

import { useState, type FormEvent } from 'react'
import { useRouter } from 'next/navigation'
import { Wrench } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useI18n } from '@/app/i18n'
import { devLogin } from '@/app/services/auth'

/** Only rendered when `GET /auth/providers` says the API mounts the dev login. */
export default function DevLoginForm() {
  const { t } = useI18n()
  const router = useRouter()
  const [login, setLogin] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [pending, setPending] = useState(false)

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    setPending(true)
    setError(null)
    try {
      await devLogin(login.trim())
      router.push('/')
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : t.login.failed)
      setPending(false)
    }
  }

  return (
    <Card className='w-full border-dashed'>
      <CardHeader>
        <CardTitle>{t.login.devTitle}</CardTitle>
        <CardDescription>{t.login.devDescription}</CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className='flex flex-col gap-4'
          onSubmit={(event) => void submit(event)}
          data-cy='dev-login-form'
        >
          <div className='flex flex-col gap-2'>
            <Label htmlFor='dev-login'>{t.login.login}</Label>
            <Input
              id='dev-login'
              value={login}
              onChange={(event) => setLogin(event.target.value)}
              autoComplete='off'
              required
              data-cy='dev-login-input'
            />
          </div>
          {error && (
            <p
              className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              role='alert'
              data-cy='dev-login-error'
            >
              {error}
            </p>
          )}
          <Button
            type='submit'
            variant='secondary'
            disabled={pending || login.trim() === ''}
            data-cy='dev-login-submit'
          >
            <Wrench />
            {t.login.devSubmit}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
