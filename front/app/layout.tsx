import '@/app/ui/global.css'
import type { Metadata } from 'next'
import { I18nProvider } from '@/app/i18n'

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
          {children}
        </I18nProvider>
      </body>
    </html>
  )
}
