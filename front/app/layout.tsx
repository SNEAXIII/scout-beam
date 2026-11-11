import '@/app/ui/global.css'
import type { Metadata } from 'next'
import { I18nProvider } from '@/app/i18n'
import LanguageSwitcher from '@/components/language-switcher'

export const metadata: Metadata = {
  title: 'Scout Beam',
  description: 'Un jeu de déduction en solo : tirez des rayons, retrouvez les pièces cachées.',
}

interface RootLayoutProps {
  readonly children: React.ReactNode
}

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html
      lang='fr'
      className='dark'
    >
      <body className='flex min-h-dvh flex-col antialiased'>
        <I18nProvider>
          <div className='flex justify-end px-4 pt-2'>
            <LanguageSwitcher />
          </div>
          {children}
        </I18nProvider>
      </body>
    </html>
  )
}
