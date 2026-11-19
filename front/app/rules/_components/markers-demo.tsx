import { useI18n } from '@/app/i18n'
import BeamMarker from '@/app/games/[id]/_components/beam-marker'
import type { ShotPath } from '@/app/games/[id]/_components/shot-paths'

const MISS: ShotPath = { key: '1-28', label: 'A', colour: 'none', from: 1, to: 28 }
const BOUNCE: ShotPath = { key: '5-31', label: 'B', colour: 'red', from: 5, to: 31 }
const RETURN: ShotPath = { key: '9-9', label: 'C', colour: 'blue', from: 9, to: 9 }

/** The three kinds of marker a shot leaves on its entries and in the shot list. */
export default function MarkersDemo() {
  const { t } = useI18n()
  const captions = t.landing.markers
  const markers = [
    { path: MISS, caption: captions.miss },
    { path: BOUNCE, caption: captions.bounce },
    { path: RETURN, caption: captions.return },
  ]
  return (
    <ul
      className='flex w-full max-w-xs flex-col gap-2 text-sm'
      data-cy='markers-demo'
    >
      {markers.map(({ path, caption }) => (
        <li
          key={path.key}
          className='flex items-center gap-3'
        >
          <BeamMarker path={path} />
          <span className='text-muted-foreground'>{caption}</span>
        </li>
      ))}
    </ul>
  )
}
