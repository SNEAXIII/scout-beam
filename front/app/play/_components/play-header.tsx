'use client'

import { Gamepad2 } from 'lucide-react'
import { useI18n } from '@/app/i18n'

export default function PlayHeader() {
  const { t } = useI18n()
  return (
    <header className='flex items-start gap-3'>
      <Gamepad2 className='mt-1 size-8 shrink-0 text-primary' />
      <div className='flex flex-col gap-1'>
        <h1 className='text-3xl font-bold tracking-tight'>{t.play.title}</h1>
        <p className='text-muted-foreground'>{t.play.subtitle}</p>
      </div>
    </header>
  )
}
