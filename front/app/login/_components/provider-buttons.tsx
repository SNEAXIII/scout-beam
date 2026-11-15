'use client'

import { Button } from '@/components/ui/button'
import { fill, useI18n } from '@/app/i18n'
import { beginOAuth, type ProviderLink } from '@/app/services/auth'
import ProviderLogo from './provider-logo'

interface ProviderButtonsProps {
  readonly providers: ProviderLink[]
  /** Kept across the provider's round trip, checked again on the way back. */
  readonly next: string | null
}

export default function ProviderButtons({ providers, next }: ProviderButtonsProps) {
  const { t } = useI18n()
  if (providers.length === 0) return null

  return (
    <div className='flex flex-col gap-2'>
      <p className='text-center text-xs uppercase text-muted-foreground'>{t.login.or}</p>
      {providers.map((provider) => (
        <Button
          key={provider.name}
          variant='outline'
          onClick={() => beginOAuth(provider, next)}
          data-cy={`provider-${provider.name}`}
        >
          <ProviderLogo name={provider.name} />
          {fill(t.login.continueWith, { provider: t.providers[provider.name] })}
        </Button>
      ))}
    </div>
  )
}
