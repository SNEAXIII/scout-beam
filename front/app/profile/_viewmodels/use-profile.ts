import { useEffect, useEffectEvent, useState } from 'react'
import { useI18n } from '@/app/i18n'
import { getSession, type UserProfile } from '@/app/services/auth'
import { rename } from '@/app/services/users'

/** The caller's profile and the pseudo being typed in its place. */
export function useProfile() {
  const { t } = useI18n()
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [draft, setDraft] = useState('')
  const [pending, setPending] = useState(false)
  const [saved, setSaved] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const failed = useEffectEvent((caught: unknown) =>
    setError(caught instanceof Error ? caught.message : t.common.loadFailed)
  )

  useEffect(() => {
    getSession().then((loaded) => {
      setProfile(loaded)
      setDraft(loaded.login)
    }, failed)
  }, [])

  const edit = (value: string) => {
    setDraft(value)
    setSaved(false)
  }

  const submit = async () => {
    setPending(true)
    setError(null)
    setSaved(false)
    try {
      const renamed = await rename(draft.trim())
      setProfile(renamed)
      setDraft(renamed.login)
      setSaved(true)
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : t.profile.failed)
    } finally {
      setPending(false)
    }
  }

  const unchanged = profile === null || draft.trim() === profile.login || draft.trim() === ''

  return { profile, draft, edit, submit, pending, saved, error, unchanged }
}

export type ProfileViewModel = ReturnType<typeof useProfile>
