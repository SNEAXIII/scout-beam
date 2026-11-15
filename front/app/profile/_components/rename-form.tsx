'use client'

import type { FormEvent } from 'react'
import { Check, Save } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useI18n } from '@/app/i18n'
import type { ProfileViewModel } from '../_viewmodels/use-profile'

export default function RenameForm({ profile }: Readonly<{ profile: ProfileViewModel }>) {
  const { t } = useI18n()
  const { draft, edit, submit, pending, saved, error, unchanged } = profile

  const onSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    void submit()
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t.profile.renameTitle}</CardTitle>
        <CardDescription>{t.profile.renameDescription}</CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className='flex flex-col gap-4'
          onSubmit={onSubmit}
          data-cy='rename-form'
        >
          <div className='flex flex-col gap-2'>
            <Label htmlFor='rename-login'>{t.profile.newLogin}</Label>
            <Input
              id='rename-login'
              value={draft}
              onChange={(event) => edit(event.target.value)}
              autoComplete='username'
              minLength={3}
              maxLength={30}
              pattern='[a-zA-Z0-9]+'
              required
              data-cy='rename-input'
            />
          </div>
          {error && (
            <p
              className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
              role='alert'
              data-cy='rename-error'
            >
              {error}
            </p>
          )}
          {saved && (
            <p
              className='flex items-center gap-2 text-sm text-muted-foreground'
              role='status'
              data-cy='rename-saved'
            >
              <Check className='size-4' />
              {t.profile.saved}
            </p>
          )}
          <Button
            type='submit'
            className='self-end'
            disabled={pending || unchanged}
            data-cy='rename-submit'
          >
            <Save />
            {t.profile.submit}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}
