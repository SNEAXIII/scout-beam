/** A beam hitting a piece in a burst of sparks; drawn in currentColor. */
export default function Logo({ className }: Readonly<{ className?: string }>) {
  return (
    <svg
      viewBox='0 0 48 48'
      fill='currentColor'
      className={className}
      aria-hidden
    >
      <rect
        x='25'
        y='14'
        width='20'
        height='20'
        rx='2'
      />
      <path
        d='M3 24h19'
        stroke='currentColor'
        strokeWidth='3'
        strokeLinecap='round'
      />
      <path d='M24.5 18.5Q24.7 23.3 26.5 24.3Q24.4 24.8 23.5 27.5Q23.4 24.5 19 23.2Q23.5 23.4 24.5 18.5Z' />
      <path d='M22.7 18.5L17.2 10.6L20.3 19.7ZM20.1 18.5L7.3 10.5L17.8 21.3ZM17 25.8L8.1 32.5L18.6 28.7ZM20.4 29L18.1 37.2L22.7 30Z' />
      <circle
        cx='14.5'
        cy='9.9'
        r='1.3'
      />
      <circle
        cx='5.5'
        cy='16.5'
        r='1.5'
      />
      <circle
        cx='9.9'
        cy='29.1'
        r='1.0'
      />
      <circle
        cx='12.3'
        cy='39'
        r='1.3'
      />
    </svg>
  )
}
