import GhostPiece from '@/components/ghost-piece'
import { useI18n } from '@/app/i18n'
import { pieceCells, type Piece, type Shape } from '@/app/lib/pieces'
import { cn } from '@/app/lib/utils'

interface TutorialCandidatesProps {
  readonly ghosts: readonly { piece: Piece; eliminated: boolean }[]
  readonly shown?: Shape
}

/** The candidate shapes over the board's top rows: the one shown is filled, a ruled-out shape crossed. */
export default function TutorialCandidates({ ghosts, shown }: TutorialCandidatesProps) {
  const { t } = useI18n()
  const shapes = [...new Set(ghosts.map((ghost) => ghost.piece.shape))]
  if (!shapes.length) return null
  return (
    <ul
      className='pointer-events-none absolute top-[8%] left-1/2 flex -translate-x-1/2 gap-1 rounded-md border bg-card/85 p-1 backdrop-blur-sm sm:gap-3 sm:p-2'
      data-cy='tutorial-candidates'
    >
      {shapes.map((shape) => {
        const { piece } = ghosts.find((ghost) => ghost.piece.shape === shape)!
        const eliminated = ghosts.every((ghost) => ghost.piece.shape !== shape || ghost.eliminated)
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
              'flex w-12 flex-col items-center gap-0.5 rounded p-1 text-[10px] sm:w-16 sm:text-xs',
              active ? 'bg-muted font-semibold' : 'text-muted-foreground'
            )}
            data-cy={`tutorial-candidate-${piece.shape}`}
            data-active={active}
          >
            <svg
              viewBox={`${left - 0.1} ${top - 0.1} ${width + 0.2} ${height + 0.2}`}
              className='size-6 sm:size-10'
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
