import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/** Where a sign-in lands: `next` when it is a path on this site, else `/play` (no open redirect). */
export function nextPath(next: string | null): string {
  if (!next?.startsWith('/')) return '/play'
  const url = new URL(next, window.location.origin)
  const path = url.pathname + url.search + url.hash
  // Dot segments can resolve to `//host`, which a router reads as another site.
  return url.origin === window.location.origin && !path.startsWith('//') ? path : '/play'
}
