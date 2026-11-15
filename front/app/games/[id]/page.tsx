'use client'

import { useEffect, useMemo, useState } from 'react'
import { useParams, useRouter, useSearchParams } from 'next/navigation'
import { useI18n } from '@/app/i18n'
import DraftPanel from './_components/draft-panel'
import GameBoard, { BOARD_COLUMN } from './_components/game-board'
import GameHeader from './_components/game-header'
import GuestSaveCard from './_components/guest-save-card'
import RoomBanner from './_components/room-banner'
import RoomLine from './_components/room-line'
import ShotList from './_components/shot-list'
import { shotPaths } from './_components/shot-paths'
import { useGame } from './_viewmodels/use-game'
import { useKeyboardLayout } from './_viewmodels/use-keyboard-layout'

export default function GamePage() {
  const { id } = useParams<{ id: string }>()
  const { t } = useI18n()
  const { game, room, error, connected, surrendering, load, fire, surrender, propose } = useGame(id)
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const { keyboardLayout, toggle } = useKeyboardLayout()
  const router = useRouter()
  const firstShot = useSearchParams().get('fire')

  // The daily's first shot, clicked on `/` before the game existed.
  useEffect(() => {
    if (!connected || firstShot === null) return
    fire(Number(firstShot))
    router.replace(`/games/${id}`)
  }, [connected, firstShot, fire, router, id])

  const shots = useMemo(() => game?.shots ?? [], [game])
  const paths = useMemo(() => shotPaths(shots), [shots])
  const board = {
    paths,
    highlighted,
    onFire: fire,
    onHighlight: setHighlighted,
  }

  return (
    <main className='mx-auto flex w-full max-w-5xl flex-col gap-3 px-3 pb-3 md:px-4'>
      <GameHeader
        game={game}
        onSurrender={surrender}
        surrendering={surrendering}
        keyboardLayout={keyboardLayout}
        onToggleKeyboardLayout={toggle}
      />
      {game?.room_code && (
        <RoomLine room={game.status === 'in_progress' ? room : { ...room, waitMs: 0 }} />
      )}
      {error && (
        <p
          className='text-sm text-destructive-foreground bg-destructive rounded-md px-3 py-2'
          role='alert'
          data-cy='game-error'
        >
          {error}
        </p>
      )}
      {game && !connected && (
        <p
          className='text-sm text-muted-foreground'
          role='status'
          data-cy='game-reconnecting'
        >
          {t.game.reconnecting}
        </p>
      )}
      {game === null && !error && (
        <p className='text-sm text-muted-foreground'>{t.common.loading}</p>
      )}
      {game && (
        <div className='grid items-start gap-4 md:grid-cols-[minmax(0,1fr)_14rem]'>
          {game.status !== 'in_progress' ? (
            <div
              className='mx-auto flex w-full flex-col gap-3'
              style={BOARD_COLUMN}
            >
              <GameBoard
                {...board}
                layout={game.layout}
                disabled
              />
              {game.room_code ? (
                <RoomBanner
                  status={game.status}
                  surrendered={room.surrendered}
                />
              ) : (
                <p
                  className='text-sm text-muted-foreground'
                  data-cy={`game-${game.status}`}
                >
                  {t.game[game.status]}
                </p>
              )}
              {game.daily_date && game.status === 'solved' && <GuestSaveCard />}
            </div>
          ) : (
            <DraftPanel
              {...board}
              gameId={id}
              keyboardLayout={keyboardLayout}
              locked={room.waitMs > 0}
              onSolved={() => void load()}
              onPropose={propose}
            />
          )}
          {/* Takes the row's height without adding to it, so the shots scroll instead of growing the row. */}
          <div className='flex min-h-0 flex-col md:h-0 md:min-h-full md:self-stretch'>
            <ShotList
              shots={shots}
              paths={paths}
              highlighted={highlighted}
              onHighlight={setHighlighted}
            />
          </div>
        </div>
      )}
    </main>
  )
}
