'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { GraduationCap } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'
import { TUTORIAL_KEY } from '@/app/tutorial/_viewmodels/use-tutorial'

/** Invites a newcomer to the Tutorial until it is finished or put off. */
export default function TutorialCard() {
  const { t } = useI18n()
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    try {
      setVisible(localStorage.getItem(TUTORIAL_KEY) === null)
    } catch {
      // Unreadable storage keeps the card hidden.
    }
  }, [])

  const later = () => {
    setVisible(false)
    try {
      localStorage.setItem(TUTORIAL_KEY, 'dismissed')
    } catch {
      // Hidden for this visit only.
    }
  }

  if (!visible) return null
  return (
    <Card data-cy='tutorial-card'>
      <CardHeader>
        <CardTitle>{t.tutorial.cardTitle}</CardTitle>
        <CardDescription>{t.tutorial.cardText}</CardDescription>
      </CardHeader>
      <CardFooter className='flex flex-wrap gap-2'>
        <Button asChild>
          <Link
            href='/tutorial'
            data-cy='tutorial-start'
          >
            <GraduationCap />
            {t.tutorial.start}
          </Link>
        </Button>
        <Button
          variant='ghost'
          onClick={later}
          data-cy='tutorial-later'
        >
          {t.tutorial.later}
        </Button>
      </CardFooter>
    </Card>
  )
}
