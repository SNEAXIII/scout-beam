'use client'

import { useState, type FormEvent } from 'react'
import { useRouter } from 'next/navigation'
import { LogIn } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useI18n } from '@/app/i18n'
import { devLogin } from '@/app/services/auth'

export default function LoginForm() {
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
    <Card className='w-full max-w-sm'>
      <CardHeader>
        <CardTitle>Scout Beam</CardTitle>
        <CardDescription>{t.login.description}</CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className='flex flex-col gap-4'
          onSubmit={(event) => void submit(event)}
          data-cy='login-form'
        >
          <div className='flex flex-col gap-2'>
            <Label htmlFor='login'>{t.login.login}</Label>
            <Input
              id='login'
              value={login}
              onChange={(event) => setLogin(event.target.value)}
              autoComplete='username'
              required
              data-cy='login-input'
            />
          </div>
          {error && (
            <p
              className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              role='alert'
              data-cy='login-error'
            >
              {error}
            </p>
          )}
          <Button
            type='submit'
            disabled={pending || login.trim() === ''}
            data-cy='login-submit'
          >
            <LogIn />
            {t.login.submit}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
