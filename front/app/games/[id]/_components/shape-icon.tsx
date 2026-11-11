import { cellPoints, pieceCells, type Rotation, type Shape } from '@/app/lib/pieces'

interface ShapeIconProps {
  readonly shape: Shape
  readonly rotation: Rotation
  readonly fill: string
}

/** A shape drawn in a 2×2 box, centred whatever its rotation. */
export default function ShapeIcon({ shape, rotation, fill }: ShapeIconProps) {
  const cells = pieceCells({ shape, colour: 'red', x: 0, y: 0, rotation })
  const width = Math.max(...cells.map((cell) => cell.x)) + 1
  const height = Math.max(...cells.map((cell) => cell.y)) + 1
  return (
    <svg
      viewBox={`${(width - 2) / 2} ${(height - 2) / 2} 2 2`}
      className='size-6'
      fill={fill}
      aria-hidden
    >
      {cells.map((cell) => (
        <polygon
          key={`${cell.x},${cell.y}`}
          points={cellPoints(cell)}
        />
      ))}
    </svg>
  )
}
