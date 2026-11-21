'use client'

import { useState } from 'react'
import { CalendarDays, Users } from 'lucide-react'
import { useI18n } from '@/app/i18n'
import BotModeCard from './bot-mode-card'
import ModeCard from './mode-card'
import RoomActions from './room-actions'

/** The three ways to play; Multijoueur unfolds the Race, Duel and room-code choices. */
export default function PlayModes() {
  const { t } = useI18n()
  const [multi, setMulti] = useState(false)
  return (
    <div className='flex flex-col gap-3'>
      <ModeCard
        icon={CalendarDays}
        title={t.play.modes.daily}
        text={t.play.dailyText}
        href='/'
        cy='daily-link'
      />
      <BotModeCard />
      <ModeCard
        icon={Users}
        title={t.play.multi}
        text={t.play.multiText}
        onClick={() => setMulti(!multi)}
        selected={multi}
        cy='multi-toggle'
      />
      {multi && <RoomActions />}
    </div>
  )
}
