import '@/app/ui/global.css'
import type { Metadata } from 'next'

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
      <body className='min-h-dvh antialiased'>{children}</body>
    </html>
  )
}
