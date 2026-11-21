'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { LogOut, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import KeyboardLayoutToggle from '@/app/games/[id]/_components/keyboard-layout-toggle'
import { useKeyboardLayout } from '@/app/games/[id]/_viewmodels/use-keyboard-layout'
import { useI18n } from '@/app/i18n'
import { logout } from '@/app/services/auth'
import LanguageSwitcher from '@/components/language-switcher'
import ThemePicker from '@/components/theme-picker'

interface ModalSettingsProps {
  readonly className: string
  readonly signedIn: boolean
}

/** Mawster's settings modal: one row per setting, label left, control right. */
export default function ModalSettings({ className, signedIn }: ModalSettingsProps) {
  const { t } = useI18n()
  const router = useRouter()
  const { keyboardLayout, toggle } = useKeyboardLayout()
  const [open, setOpen] = useState(false)

  // Off to /login even when the API is unreachable: its cookies then expire on their own.
  const signOut = () => {
    setOpen(false)
    void logout()
      .catch(() => undefined)
      .finally(() => router.replace('/login'))
  }

  const rows = [
    { label: t.nav.language, control: <LanguageSwitcher /> },
    { label: t.nav.theme, control: <ThemePicker /> },
    {
      label: t.nav.keyboard,
      control: (
        <KeyboardLayoutToggle
          keyboardLayout={keyboardLayout}
          onToggle={toggle}
        />
      ),
    },
  ]
  if (signedIn)
    rows.push({
      label: t.nav.signOut,
      control: (
        <Button
          type='button'
          variant='destructive'
          onClick={signOut}
          className='px-2 hover:bg-destructive/30 hover:text-destructive'
          aria-label={t.nav.signOut}
          data-cy='sign-out'
        >
          <LogOut className='size-5' />
        </Button>
      ),
    })

  return (
    <Dialog
      open={open}
      onOpenChange={setOpen}
    >
      <DialogTrigger
        className={className}
        aria-label={t.nav.settings}
        title={t.nav.settings}
        data-cy='modal-settings-trigger'
      >
        <Settings className='size-6' />
      </DialogTrigger>
      <DialogContent data-cy='modal-settings-content'>
        <DialogHeader>
          <DialogTitle>{t.nav.settings}</DialogTitle>
        </DialogHeader>
        <div className='mt-4 divide-y'>
          {rows.map(({ label, control }) => (
            <div
              key={label}
              className='flex items-center justify-between py-3'
            >
              <span className='text-sm font-medium'>{label}</span>
              {control}
            </div>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  )
}
