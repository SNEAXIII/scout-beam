'use client'

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react'
import en, { type Translations } from './locales/en'
import fr from './locales/fr'
import { ApiError } from '@/app/lib/api'

export type Locale = 'en' | 'fr'

const locales: Record<Locale, Translations> = { en, fr }

const STORAGE_KEY = 'scout-beam-locale'
const DEFAULT_LOCALE: Locale = 'fr'

interface I18nContextType {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: Translations
}

const I18nContext = createContext<I18nContextType>({
  locale: DEFAULT_LOCALE,
  setLocale: () => {},
  t: fr,
})

export function I18nProvider({ children }: Readonly<{ children: React.ReactNode }>) {
  const [locale, setLocale] = useState<Locale>(DEFAULT_LOCALE)

  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'en' || stored === 'fr') setLocale(stored)
  }, [])

  useEffect(() => {
    document.documentElement.lang = locale
  }, [locale])

  const changeLocale = useCallback((next: Locale) => {
    setLocale(next)
    localStorage.setItem(STORAGE_KEY, next)
  }, [])

  const value = useMemo(
    () => ({ locale, setLocale: changeLocale, t: locales[locale] }),
    [locale, changeLocale]
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

export function useI18n() {
  return useContext(I18nContext)
}

/** Fills every `{{key}}` of a translation. */
export const fill = (template: string, values: Record<string, string | number>) =>
  template.replaceAll(/\{\{(\w+)\}\}/g, (_, key: string) => String(values[key]))

/** A caught error in the player's language: its API `code` translated, else its own message. */
export function errorText(t: Translations, caught: unknown, fallback = ''): string {
  if (!(caught instanceof ApiError)) return caught instanceof Error ? caught.message : fallback
  const errors: Partial<Record<string, string>> = t.errors
  const template = errors[caught.code]
  if (template === undefined) return caught.message
  const shapes: Partial<Record<string, string>> = t.shapes
  const colours: Partial<Record<string, string>> = t.colours
  const params = Object.entries(caught.params).map(([key, value]) => [
    key,
    shapes[value] ?? colours[value]?.toLowerCase() ?? value,
  ])
  return fill(template, Object.fromEntries(params) as Record<string, string | number>)
}

export { type Translations }
