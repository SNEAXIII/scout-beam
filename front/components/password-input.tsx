'use client'

import { useState, type ComponentProps } from 'react'
import { Eye, EyeOff } from 'lucide-react'
import { Input } from '@/components/ui/input'
import { useI18n } from '@/app/i18n'

/** A password field with an eye to show what was typed. */
export default function PasswordInput(props: Readonly<Omit<ComponentProps<typeof Input>, 'type'>>) {
  const { t } = useI18n()
  const [shown, setShown] = useState(false)
  const label = shown ? t.login.hidePassword : t.login.showPassword

  return (
    <div className='relative'>
      <Input
        {...props}
        type={shown ? 'text' : 'password'}
        className='pr-10'
      />
      <button
        type='button'
        onClick={() => setShown(!shown)}
        className='absolute inset-y-0 right-0 flex w-10 items-center justify-center text-muted-foreground hover:text-foreground'
        aria-label={label}
        aria-pressed={shown}
        title={label}
        data-cy={`${props.id}-toggle`}
      >
        {shown ? <EyeOff className='size-4' /> : <Eye className='size-4' />}
      </button>
    </div>
  )
}
