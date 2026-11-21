# Piece wheel

**Goal:** a radial item wheel, held open with `S` or a long press on an empty cell, that picks the
draft's colour and shape by aiming at a slice and releasing.

## Global Constraints

- Front only, `front/app/`; check with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware`
- Every string through `useI18n()`, keys in both `en.ts` and `fr.ts`; colours from `PIECE_SWATCHES`
- Files ≤150 lines; `data-cy` on the wheel and its slices; icons from `lucide-react`; no new dependency
- The wheel only changes the draft's picked state through `Draft.selectColour` / `Draft.selectShape`
  (`games/[id]/_viewmodels/use-draft.ts`): it never places a piece, and a click on a cell still places
  as today. The toolbar stays as it is and shows every change.
- Slice order, both rings: first slice centred straight up, then clockwise — colours in
  `PIECE_COLOURS` order, shapes in `SHAPES` order (all five, placed ones included)

---

### Task 1: Wheel geometry

**Files:** `front/app/lib/wheel.ts` (new), `front/app/lib/wheel.check.ts` (new), `front/package.json`

Pure module, no React. Export `WHEEL_RADIUS = 120` (px) and the ring bounds as fractions of it:
centre (dead zone) up to `0.25`, colour ring `0.25`–`0.55`, shape ring `0.55`–`1`. Export
`wheelSliceAt(dx, dy)`: the pointer's offset from the wheel centre in px (y downwards) to
`{ ring: 'colour', index }` (4 slices of 90°), `{ ring: 'shape', index }` (5 slices of 72°), or
`null` in the dead zone and beyond the radius. Also export `slicePath(ring, index)` returning the
SVG `d` of that slice (an annulus sector, centre at 0,0) and `sliceCentre(ring, index)` returning
its `{ x, y }` midpoint, so Task 2 draws exactly what `wheelSliceAt` hits.

`wheel.check.ts` follows `app/tutorial/_lib/script.check.ts` (`node:assert/strict`, imports with
`.ts`): asserts straight up in each ring hits index 0, a point just clockwise of a boundary hits
the next index, the dead zone and outside return `null`, and every `sliceCentre` maps back to its
own slice. Add the script `"check:wheel": "node app/lib/wheel.check.ts"` to `front/package.json`.

**Done when:** `pnpm check:wheel` passes and fails if a ring bound or the start angle changes.

### Task 2: The wheel and its triggers

**Files:** `front/app/games/[id]/_components/piece-wheel.tsx` (new),
`front/app/games/[id]/_viewmodels/use-piece-wheel.ts` (new),
`front/app/games/[id]/_components/draft-board.tsx`, `front/app/games/[id]/_viewmodels/use-long-press.ts`,
`front/app/lib/keyboard.ts`, `front/app/games/[id]/_components/hotkey-legend.tsx`,
`front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

Uses `WHEEL_RADIUS`, `wheelSliceAt`, `slicePath`, `sliceCentre` from `@/app/lib/wheel` (Task 1).

**State** (`usePieceWheel(draft)`, mounted from `DraftBoard` so the game, tutorial and duel compose
boards all get it): `open: { x, y } | null` (viewport px of the centre), `hovered` slice. Opening
remembers nothing else. While open, every `pointermove` on `window` computes `wheelSliceAt` from
the centre: entering a colour slice calls `draft.selectColour` at once; a shape slice is only
highlighted. Releasing calls `draft.selectShape` if a shape slice is hovered (a placed shape is
refused by `selectShape` already), then closes. Nothing else happens on release.

**Desktop trigger.** Add `wheel: 's'` to both layouts in `KEYMAPS` (`lib/keyboard.ts`). `keydown`
of that key (no repeat, no modifier, not while typing — reuse the `typing` check of `use-draft.ts`,
export it) opens the wheel at the last pointer position over the board, or the board's centre if
the pointer never moved over it; `keyup` releases. While open, `R` keeps rotating (already in
`use-draft.ts`) and a `wheel` event (non-passive, `preventDefault`) rotates once per event with
`draft.rotate`. Blur of the window closes without selecting.

**Touch trigger.** A long press (reuse `LONG_PRESS_MS` and `SLOP_PX` from `use-long-press.ts`,
export them) on an empty draft cell opens the wheel centred on the press; sliding the finger aims,
lifting releases. Page scroll on the grid keeps working until the wheel opens; once open, a
non-passive `touchmove` listener on `window` calls `preventDefault`. The click the lift may
synthesise must not place a piece: reuse `swallowNextClick` from `use-long-press.ts` (export it).
Long press on a placed piece still removes it (untouched).

**Drawing** (`PieceWheel`): `position: fixed`, centred on `open`, the centre clamped so the
`WHEEL_RADIUS` circle stays in the viewport (and `wheelSliceAt` measured from the clamped centre),
`pointer-events: none`, `z-50`. One SVG of `2 × WHEEL_RADIUS`: each slice a `slicePath` in
`bg-card`-like fill with a `border` stroke, the hovered one in `accent`; colour slices filled with
their swatch, the selected colour outlined in `foreground`; on each shape slice a `ShapeIcon`
(`@/components/shape-icon`) at `sliceCentre`, in the current colour and rotation, placed shapes at
40% opacity. Centre shows the hovered slice's name (`t.shapes` / `t.colours`), or `t.draft.wheel`.
`data-cy='piece-wheel'`, slices `data-cy='wheel-colour-<colour>'` / `'wheel-shape-<shape>'` with
`data-hovered`.

**Legend and i18n.** `HotkeyLegend` gains `{ keys: [keys.wheel], label: t.hotkeys.wheel }`; the
touch legend line gains `t.hotkeys.holdForWheel`. Keys: `draft.wheel` (en `'Pick a piece'`, fr
`'Choisir une pièce'`), `hotkeys.wheel` (en `'hold: piece wheel'`, fr `'maintenir : roue des
pièces'`), `hotkeys.holdForWheel` (en `'Hold an empty cell to open the piece wheel'`, fr
`'Maintiens une case vide pour ouvrir la roue des pièces'`).

**Done when:** in a bot game, holding `S` shows the wheel at the pointer; sweeping over blue then
releasing on the bar leaves the toolbar on a blue bar, and a click places it; releasing in the
centre changes only the colour passed over; the scroll wheel and `R` turn the shape icons and the
toolbar rotation; on a touch viewport (devtools) a long press on an empty cell opens it, slide and
lift picks, no piece is placed by the lift, a long press on a piece still removes it; the tutorial
and duel compose board behave the same; `tsc`, `oxlint` and `pnpm check:wheel` pass.
