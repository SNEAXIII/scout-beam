'use client'

import { useState } from 'react'
import { Check, Link2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useI18n } from '@/app/i18n'

type Copy = 'idle' | 'copied' | 'failed'

/** The code, and the invitation link to hand the opponent. */
export default function RoomCode({ code }: Readonly<{ code: string }>) {
  const { t } = useI18n()
  const [copy, setCopy] = useState<Copy>('idle')

  // The clipboard is refused outside a secure context: the page's address is then the fallback.
  const copyLink = () => {
    navigator.clipboard.writeText(`${window.location.origin}/rooms/${code}`).then(
      () => setCopy('copied'),
      () => setCopy('failed')
    )
  }

  return (
    <div className='flex flex-col items-center gap-3'>
      <p
        className='font-mono text-4xl font-semibold tracking-[0.3em]'
        data-cy='room-code'
      >
        {code}
      </p>
      <Button
        variant='outline'
        onClick={copyLink}
        data-cy='room-copy-link'
      >
        {copy === 'copied' ? <Check /> : <Link2 />}
        {copy === 'copied' ? t.rooms.copied : t.rooms.copyLink}
      </Button>
      {copy === 'failed' && (
        <p
          className='text-sm text-muted-foreground'
          role='alert'
          data-cy='room-copy-failed'
        >
          {t.rooms.copyFailed}
        </p>
      )}
    </div>
  )
}
