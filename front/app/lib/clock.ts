const pad = (value: number) => String(value).padStart(2, '0')

/** `mm:ss`, or `h:mm:ss` from an hour on. */
export function clock(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const rest = `${pad(Math.floor(seconds / 60) % 60)}:${pad(seconds % 60)}`
  return hours > 0 ? `${hours}:${rest}` : rest
}
