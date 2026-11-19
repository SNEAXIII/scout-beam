import Link from 'next/link'
import { BookOpen, Play } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useI18n } from '@/app/i18n'

export default function TutorialDone() {
  const { t } = useI18n()
  return (
    <Card data-cy='tutorial-done'>
      <CardHeader>
        <CardTitle>{t.tutorial.doneTitle}</CardTitle>
      </CardHeader>
      <CardContent className='flex flex-col gap-4'>
        <p className='text-sm text-muted-foreground'>{t.tutorial.doneText}</p>
        <div className='flex flex-wrap gap-2'>
          <Button asChild>
            <Link
              href='/'
              data-cy='tutorial-daily'
            >
              <Play />
              {t.tutorial.playDaily}
            </Link>
          </Button>
          <Button
            asChild
            variant='outline'
          >
            <Link
              href='/rules'
              data-cy='tutorial-rules'
            >
              <BookOpen />
              {t.tutorial.readRules}
            </Link>
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}
