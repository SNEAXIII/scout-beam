'use client'

import { useEffect, useState } from 'react'
import { usePathname } from 'next/navigation'
import { peekSession, type UserProfile } from '@/app/services/auth'

/** undefined while unknown, null when signed out. Read again on each route: signing in or out navigates. */
export function useNavUser(): UserProfile | null | undefined {
  const path = usePathname()
  const [user, setUser] = useState<UserProfile | null | undefined>(undefined)

  useEffect(() => {
    let live = true
    peekSession().then(
      (session) => live && setUser(session),
      // API unreachable: keep what was known rather than flash the signed-out nav.
      () => undefined
    )
    return () => {
      live = false
    }
  }, [path])

  return user
}
