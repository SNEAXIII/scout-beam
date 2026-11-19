import GhostPiece from '@/components/ghost-piece'
import { useI18n } from '@/app/i18n'
import { pieceCells, type Piece, type Shape } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'

interface TutorialCandidatesProps {
  readonly ghosts: readonly { piece: Piece; eliminated: boolean }[]
  readonly shown?: Shape
}

/** The candidate shapes side by side: the one on the board is filled, the ruled-out ones crossed. */
export default function TutorialCandidates({ ghosts, shown }: TutorialCandidatesProps) {
  const { t } = useI18n()
  if (!ghosts.length) return null
  return (
    <ul
      className='flex justify-center gap-3 rounded-md border bg-card p-2'
      data-cy='tutorial-candidates'
    >
      {ghosts.map(({ piece, eliminated }) => {
        const cells = pieceCells(piece)
        const left = Math.min(...cells.map((cell) => cell.x))
        const top = Math.min(...cells.map((cell) => cell.y))
        const width = Math.max(...cells.map((cell) => cell.x)) + 1 - left
        const height = Math.max(...cells.map((cell) => cell.y)) + 1 - top
        const active = piece.shape === shown && !eliminated
        return (
          <li
            key={piece.shape}
            className={cn(
              'flex w-16 flex-col items-center gap-1 rounded p-1 text-xs',
              active ? 'bg-muted font-semibold' : 'text-muted-foreground'
            )}
            data-cy={`tutorial-candidate-${piece.shape}`}
            data-active={active}
          >
            <svg
              viewBox={`${left - 0.1} ${top - 0.1} ${width + 0.2} ${height + 0.2}`}
              className='size-10'
              aria-hidden
            >
              <GhostPiece
                piece={piece}
                eliminated={eliminated}
                highlighted={active}
              />
            </svg>
            <span className={cn(eliminated && 'line-through')}>{t.shapes[piece.shape]}</span>
          </li>
        )
      })}
    </ul>
  )
}
