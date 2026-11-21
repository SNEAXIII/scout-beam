'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import {
  BookOpen,
  Gamepad2,
  GraduationCap,
  House,
  LogIn,
  Trophy,
  UserRound,
  type LucideIcon,
} from 'lucide-react'
import { useI18n, type Translations } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import Logo from '@/components/logo'
import ModalSettings from './modal-settings'
import { useNavUser } from './use-nav-user'

interface NavEntry {
  readonly href: string
  readonly icon: LucideIcon
  readonly label: keyof Translations['nav']
  /** Other routes that light this entry. */
  readonly also?: readonly string[]
  /** true: signed in only (guests included); false: signed out only; absent: always. */
  readonly signedIn?: boolean
}

const ENTRIES: readonly NavEntry[] = [
  { href: '/', icon: House, label: 'home' },
  { href: '/play', icon: Gamepad2, label: 'play', also: ['/games', '/rooms'], signedIn: true },
  { href: '/ranking', icon: Trophy, label: 'ranking' },
  { href: '/tutorial', icon: GraduationCap, label: 'tutorial' },
  { href: '/rules', icon: BookOpen, label: 'rules' },
  { href: '/profile', icon: UserRound, label: 'profile', signedIn: true },
  { href: '/login', icon: LogIn, label: 'signIn', also: ['/register'], signedIn: false },
]

const ITEM =
  'flex h-12 w-full max-w-14 items-center justify-center rounded-xl transition-colors md:size-12'
// Seven slots share a 360px phone: each takes an equal part, up to 3.5rem.
const SLOT = 'flex min-w-0 flex-1 justify-center md:flex-none'
const IDLE = 'text-foreground hover:bg-accent'

const under = (path: string, href: string) =>
  href === '/' ? path === '/' : path === href || path.startsWith(href + '/')

const isActive = (path: string, { href, also = [] }: NavEntry) =>
  [href, ...also].some((prefix) => under(path, prefix))

/** Icon rail from md, bottom tabs below it; the tabs leave a game to its own header. */
export default function AppNav() {
  const { t } = useI18n()
  const path = usePathname()
  const inGame = path.startsWith('/games/')
  const user = useNavUser()
  // While the session is unknown, neither the signed-in nor the signed-out entries show.
  const visible = ENTRIES.filter(
    ({ signedIn }) => signedIn === undefined || (user !== undefined && signedIn === (user !== null))
  )

  return (
    <nav
      aria-label={t.nav.label}
      className={cn(
        'fixed inset-x-0 bottom-0 z-40 border-t bg-sidebar pb-[env(safe-area-inset-bottom)]',
        'md:inset-y-0 md:right-auto md:w-20 md:border-t-0 md:border-r md:pb-0',
        inGame && 'max-md:hidden'
      )}
      data-cy='app-nav'
    >
      <ul className='flex h-16 items-center px-1 md:h-full md:flex-col md:justify-start md:gap-3 md:py-4'>
        <li className='hidden md:mb-4 md:block'>
          <Link
            href='/'
            aria-label='Scout Beam'
            className='flex size-12 items-center justify-center text-primary'
          >
            <Logo className='size-10' />
          </Link>
        </li>
        {visible.map((entry) => {
          const active = isActive(path, entry)
          const Icon = entry.icon
          return (
            <li
              key={entry.href}
              className={SLOT}
            >
              <Link
                href={entry.href}
                aria-label={t.nav[entry.label]}
                title={t.nav[entry.label]}
                aria-current={active ? 'page' : undefined}
                // Sign in stands out, as in Mawster.
                className={cn(
                  ITEM,
                  active || entry.signedIn === false ? 'bg-primary text-primary-foreground' : IDLE
                )}
                data-cy={`nav-${entry.label}`}
              >
                <Icon className='size-6' />
              </Link>
            </li>
          )
        })}
        <li className={cn(SLOT, 'md:mt-auto')}>
          <ModalSettings
            className={cn(ITEM, IDLE)}
            signedIn={Boolean(user)}
          />
        </li>
      </ul>
    </nav>
  )
}
