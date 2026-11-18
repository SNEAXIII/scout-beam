'use client'

import type { ReactNode } from 'react'
import { useI18n } from '@/app/i18n'
import { cn } from '@/app/lib/utils'
import type { Ranking, RankingEntry } from '@/app/services/daily'

interface RankingTableProps<T extends RankingEntry> {
  /** Null while loading. */
  readonly ranking: Ranking<T> | null
  /** Headers of the columns after rank and login. */
  readonly headers: readonly string[]
  readonly cells: (entry: T) => readonly ReactNode[]
  readonly cy: string
  readonly rowClass?: (entry: T) => string | false
}

/** Rank and login, then the given columns; the caller's line highlighted, pinned below when outside the top. */
export default function RankingTable<T extends RankingEntry>({
  ranking,
  headers,
  cells,
  cy,
  rowClass,
}: RankingTableProps<T>) {
  const { t } = useI18n()
  if (ranking === null) return <p className='text-sm text-muted-foreground'>{t.common.loading}</p>
  const { top, me } = ranking
  if (top.length === 0) {
    return (
      <p
        className='text-sm text-muted-foreground'
        data-cy={`${cy}-empty`}
      >
        {t.ranking.empty}
      </p>
    )
  }

  const row = (entry: T, pinned: boolean) => (
    <tr
      key={pinned ? 'me' : entry.login}
      className={cn(
        'border-t',
        entry.login === me?.login && 'bg-accent text-accent-foreground',
        rowClass?.(entry)
      )}
      data-cy={pinned ? `${cy}-me` : `${cy}-row`}
    >
      <td className='px-3 py-2 tabular-nums'>{entry.rank}</td>
      <td className='px-3 py-2 font-medium'>{entry.login}</td>
      {cells(entry).map((cell, index) => (
        <td
          key={headers[index]}
          className='px-3 py-2 text-right tabular-nums'
        >
          {cell}
        </td>
      ))}
    </tr>
  )

  const pinned = me !== null && !top.some((entry) => entry.login === me.login)
  return (
    <table
      className='w-full text-sm'
      data-cy={cy}
    >
      <thead className='text-muted-foreground'>
        <tr>
          <th className='px-3 py-2 text-left font-normal'>{t.ranking.rank}</th>
          <th className='px-3 py-2 text-left font-normal'>{t.ranking.login}</th>
          {headers.map((header) => (
            <th
              key={header}
              className='px-3 py-2 text-right font-normal'
            >
              {header}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {top.map((entry) => row(entry, false))}
        {pinned && (
          <>
            <tr aria-hidden>
              <td
                colSpan={headers.length + 2}
                className='px-3 text-muted-foreground'
              >
                …
              </td>
            </tr>
            {row(me, true)}
          </>
        )}
      </tbody>
    </table>
  )
}
