import { useCallback, useEffect, useState } from 'react'
import { useI18n } from '@/app/i18n'
import type { KeyboardLayout } from '@/app/lib/keyboard'

const STORAGE_KEY = 'scout-beam.keyboard-layout'

/** Follows the interface language until the player picks a keyboard layout. */
export function useKeyboardLayout() {
  const { locale } = useI18n()
  const [chosen, setChosen] = useState<KeyboardLayout | null>(null)

  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored === 'azerty' || stored === 'qwerty') setChosen(stored)
    } catch {
      // Storage unavailable: keep following the language.
    }
  }, [])

  const keyboardLayout: KeyboardLayout = chosen ?? (locale === 'fr' ? 'azerty' : 'qwerty')

  const toggle = useCallback(() => {
    const next: KeyboardLayout = keyboardLayout === 'azerty' ? 'qwerty' : 'azerty'
    setChosen(next)
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // Storage unavailable: the choice lasts until the page is left.
    }
  }, [keyboardLayout])

  return { keyboardLayout, toggle }
}
