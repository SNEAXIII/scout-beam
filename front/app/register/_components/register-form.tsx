'use client'

import { useState, type FormEvent } from 'react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { UserPlus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useI18n } from '@/app/i18n'
import { register } from '@/app/services/auth'

interface FieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'password'
  autoComplete: string
}

function Field({ id, label, value, onChange, type = 'text', autoComplete }: Readonly<FieldProps>) {
  return (
    <div className='flex flex-col gap-2'>
      <Label htmlFor={id}>{label}</Label>
      <Input
        id={id}
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        autoComplete={autoComplete}
        required
        data-cy={`${id}-input`}
      />
    </div>
  )
}

/** A guest's session is converted by `register`: same account, games kept. */
export default function RegisterForm() {
  const { t } = useI18n()
  const router = useRouter()
  const [login, setLogin] = useState('')
  const [password, setPassword] = useState('')
  const [confirmation, setConfirmation] = useState('')
  const [error, setError] = useState<string | null>(null)
  const [pending, setPending] = useState(false)

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (password !== confirmation) {
      setError(t.register.mismatch)
      return
    }
    setPending(true)
    setError(null)
    try {
      await register(login.trim(), password)
      router.push('/')
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : t.register.failed)
      setPending(false)
    }
  }

  return (
    <Card className='w-full max-w-sm'>
      <CardHeader>
        <CardTitle>{t.register.title}</CardTitle>
        <CardDescription>{t.register.description}</CardDescription>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <form
          className='flex flex-col gap-4'
          onSubmit={(event) => void submit(event)}
          data-cy='register-form'
        >
          <Field
            id='register-login'
            label={t.login.login}
            value={login}
            onChange={setLogin}
            autoComplete='username'
          />
          <Field
            id='register-password'
            label={t.login.password}
            value={password}
            onChange={setPassword}
            type='password'
            autoComplete='new-password'
          />
          <Field
            id='register-confirmation'
            label={t.register.confirm}
            value={confirmation}
            onChange={setConfirmation}
            type='password'
            autoComplete='new-password'
          />
          {error && (
            <p
              className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              role='alert'
              data-cy='register-error'
            >
              {error}
            </p>
          )}
          <Button
            type='submit'
            disabled={pending || login.trim() === '' || password === ''}
            data-cy='register-submit'
          >
            <UserPlus />
            {t.register.submit}
          </Button>
        </form>
        <p className='text-center text-sm text-muted-foreground'>
          {t.register.haveAccount}{' '}
          <Link
            href='/login'
            className='text-primary underline-offset-4 hover:underline'
            data-cy='login-link'
          >
            {t.register.signIn}
          </Link>
        </p>
      </CardContent>
    </Card>
  )
}
