'use client'

import { useI18n } from '@/app/i18n'

const ORAPA_MINE_URL = 'https://boardgamegeek.com/boardgame/424152/orapa-mine'

export default function InspirationSection() {
  const { t } = useI18n()
  return (
    <footer
      className='border-t pt-6 text-center text-sm text-muted-foreground'
      data-cy='inspiration'
    >
      <p>
        {t.landing.inspiredBy}
        <a
          href={ORAPA_MINE_URL}
          target='_blank'
          rel='noopener noreferrer'
          className='italic text-foreground underline underline-offset-4 hover:text-primary'
          data-cy='inspiration-link'
        >
          {t.landing.game}
        </a>
        {t.landing.credit}
      </p>
    </footer>
  )
}
