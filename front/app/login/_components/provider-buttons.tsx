'use client'

import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import { beginOAuth, type ProviderLink } from '@/app/services/auth'

/** Brand logos are absent from lucide-react: each button carries the provider's name only. */
export default function ProviderButtons({ providers }: Readonly<{ providers: ProviderLink[] }>) {
  const { t } = useI18n()
  if (providers.length === 0) return null

  return (
    <div className='flex flex-col gap-2'>
      <p className='text-center text-xs uppercase text-muted-foreground'>{t.login.or}</p>
      {providers.map((provider) => (
        <Button
          key={provider.name}
          variant='outline'
          onClick={() => beginOAuth(provider)}
          data-cy={`provider-${provider.name}`}
        >
          {fill(t.login.continueWith, { provider: t.providers[provider.name] })}
        </Button>
      ))}
    </div>
  )
}
