import type { BeamColour } from '@/app/lib/colours'
import type { Shot } from '@/app/services/games'

/** The two ends a beam joins, as first fired; firing the other end answers the same path. */
export interface ShotPath {
  readonly key: string
  readonly label: string
  readonly colour: BeamColour
  readonly from: number
  readonly to: number
}

export interface ShotPaths {
  readonly ofShot: (shot: Shot) => ShotPath
  /** The engine is reversible: an entry, fired end or exit, lies on one path at most. */
  readonly atEntry: (entry: number) => ShotPath | undefined
  readonly fired: ReadonlySet<number>
}

const pathKey = (shot: Shot): string =>
  [shot.entry, shot.exit_entry].sort((a, b) => a - b).join('-')

/** A, B, … Z, then AA, AB, … — the 36 entries need at most 36 labels. */
function label(index: number): string {
  const letter = (i: number) => String.fromCodePoint(65 + i)
  return index < 26 ? letter(index) : letter(Math.floor(index / 26) - 1) + letter(index % 26)
}

/** Labels paths in the order their first shot was fired. */
export function shotPaths(shots: readonly Shot[]): ShotPaths {
  const byKey = new Map<string, ShotPath>()
  const byEntry = new Map<number, ShotPath>()
  for (const shot of shots) {
    const key = pathKey(shot)
    if (byKey.has(key)) continue
    const path: ShotPath = {
      key,
      label: label(byKey.size),
      colour: shot.colour,
      from: shot.entry,
      to: shot.exit_entry,
    }
    byKey.set(key, path)
    byEntry.set(path.from, path)
    byEntry.set(path.to, path)
  }
  return {
    ofShot: (shot) => byKey.get(pathKey(shot)) as ShotPath,
    atEntry: (entry) => byEntry.get(entry),
    fired: new Set(shots.map((shot) => shot.entry)),
  }
}
