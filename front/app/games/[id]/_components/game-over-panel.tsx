import { useI18n } from '@/app/i18n'
import DayRanking from '@/app/_components/day-ranking'
import { sameLook, type Piece } from '@/app/lib/pieces'
import type { GameStatus, GameView } from '@/app/services/games'
import GameBoard, { BOARD_COLUMN, type GameBoardProps } from './game-board'
import GuestSaveCard from './guest-save-card'
import RoomBanner from './room-banner'
import TraceLines from './trace-lines'
import TraceModeToggle from './trace-mode-toggle'
import type { RoomGame } from '../_viewmodels/use-room-game'
import { useTraceMode } from '../_viewmodels/use-trace-mode'
import { drawnTraces, type Traces } from '../_viewmodels/use-traces'

interface GameOverPanelProps extends Omit<
  GameBoardProps,
  'layout' | 'draft' | 'disabled' | 'traceKeyAt' | 'onSelect' | 'children'
> {
  readonly game: GameView
  readonly status: Exclude<GameStatus, 'in_progress'>
  readonly room: RoomGame
  readonly proposed: readonly Piece[] | null
  readonly traces: Traces
}

/** The revealed layout, where clicking an entry draws its trace, and how the game ended. */
export default function GameOverPanel({
  game,
  status,
  room,
  proposed,
  traces,
  ...board
}: GameOverPanelProps) {
  const { t } = useI18n()
  const { layout } = game
  const { mode, choose } = useTraceMode()

  return (
    <div
      className='mx-auto flex w-full flex-col gap-3'
      style={BOARD_COLUMN}
    >
      <GameBoard
        {...board}
        layout={layout}
        disabled
        traceKeyAt={traces.keyAt}
        onSelect={layout && traces.toggle}
      >
        <TraceLines
          traces={drawnTraces(traces, mode, board.highlighted)}
          animated={mode === 'selected' ? traces.selected : null}
        />
      </GameBoard>
      <div className='flex items-start gap-3'>
        <div className='min-w-0 flex-1'>
          {game.room_code ? (
            <RoomBanner
              status={status}
              surrendered={room.surrendered}
            />
          ) : (
            <p
              className='text-sm text-muted-foreground'
              data-cy={`game-${status}`}
            >
              {t.game[status]}
            </p>
          )}
        </div>
        {layout && (
          <TraceModeToggle
            mode={mode}
            onChange={choose}
          />
        )}
      </div>
      {status === 'solved' && proposed && layout && !sameLook(proposed, layout.pieces) && (
        <p
          className='text-sm text-muted-foreground'
          data-cy='game-equivalent'
        >
          {t.game.equivalent}
        </p>
      )}
      {game.daily_date && status === 'solved' && <GuestSaveCard />}
      {game.daily_date && <DayRanking date={game.daily_date} />}
    </div>
  )
}
