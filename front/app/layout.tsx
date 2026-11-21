import '@/app/ui/global.css'
import type { Metadata } from 'next'
import { I18nProvider } from '@/app/i18n'
import { THEME_SCRIPT } from '@/app/lib/theme-script'
import AppNav from '@/components/nav/app-nav'

export const metadata: Metadata = {
  title: 'Scout Beam',
  description: 'Un jeu de déduction : tirez des rayons, retrouvez les pièces cachées.',
}

interface RootLayoutProps {
  readonly children: React.ReactNode
}

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html
      lang='fr'
      className='dark'
      suppressHydrationWarning
    >
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_SCRIPT }} />
      </head>
      <body className='flex min-h-dvh flex-col pb-[calc(4rem+env(safe-area-inset-bottom))] antialiased md:pb-0 md:pl-20 has-[[data-game]]:max-md:pb-0'>
        <I18nProvider>
          <AppNav />
          {children}
        </I18nProvider>
      </body>
    </html>
  )
}
