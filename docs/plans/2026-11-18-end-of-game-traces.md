# End-of-game traces

**Goal:** once a game is over, clicking an entry or a shot draws its Trace on the revealed layout,
and a toggle switches between the selected trace only and the traces of all 36 entries.

## Global Constraints

- Work in `front/`, on branch `ccr-1f69d5c3-92p0um`; follow `CLAUDE.md` and the vocabulary of
  `CONTEXT.md` (**Trace**, **Entry**, **Beam colour**, **Layout**).
- The API does not change: traces are computed in the front from the revealed `game.layout`.
- No new dependency. `front/node_modules` is absent: run `pnpm install` in `front/` first.
- Check with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware` from `front/`.
- Never edit `components/ui/` (except `button.tsx`, `card.tsx`); no `h-*` on a `Button`.
- No hardcoded string: `useI18n()`, keys in both `i18n/locales/en.ts` and `fr.ts`, French in the
  same register as the rest of `fr.ts`. Files ≤150 lines. Keep every existing `data-cy`.
- A trace is drawn in **one colour, the beam's final colour** (`BEAM_SWATCHES[colour]`), never
  segment by segment.
- To see the result: `pnpm dev` (port 3001) and a Playwright script in the session scratchpad
  that mocks the API (`GET /games/<id>` answering a finished game with a `layout` and some
  `shots`), screenshots at 1366×700 and 390×844.

---

### Task 1: Trace computation, ported from the engine

**Files:** `front/app/lib/trace.ts`

Port `trace` from `api/src/engine/beam.rs` with what it uses: `Entry::launch`, `Entry::leaving`
and `Cell::step` (`grid.rs`), `CellFill::deflect` (`piece.rs`), `BeamColour::mix` (`colour.rs`),
and the occupancy built from `pieceCells` of `lib/pieces.ts` (cells outside the grid ignored).
Same step cap as the engine (4 × columns × rows); past it, return `undefined` rather than throw.
Use relative imports (`./pieces`, `./colours`), as `entries.ts` does.

Export `traceBeam(layout, entry)` returning `{ entry, exit, colour, points }`, where `points` is
the polyline in grid units (one unit per cell, origin at the grid's top-left, as `GridSvg` draws):

- it starts on the grid border in the middle of the entry's row or column, and ends the same way
  at the exit;
- a bounce off a straight side (a full cell, or the straight side of a half cell) turns back on
  the middle of the cell side it came in through;
- a turn on a slanted side bends at the cell's centre;
- consecutive collinear points are merged.

Verify with a throwaway script in the session scratchpad (not committed), run with Node 22's
`--experimental-strip-types` and a resolve hook adding `.ts` to extensionless imports. It
replays every case of `beam.rs`'s tests: empty grid (each entry to its opposite, colour `none`),
the full square sending 5, 14, 24 and 33 back blue, the 16-row spike table, spike + square from
5 back to 5 violet, the diamond from 33 to 5 white, the square from 1 to 28 with colour `none`.
It also checks, for one case of each kind, that the `points` match the rules above.

**Done when:** the script passes every case, and `tsc` and `oxlint` are clean.

### Task 2: Selected trace on a finished game

**Files:** `front/app/games/[id]/_viewmodels/use-traces.ts`,
`front/app/games/[id]/_components/trace-lines.tsx`, `front/app/games/[id]/_components/grid-svg.tsx`,
`front/app/games/[id]/_components/game-board.tsx`,
`front/app/games/[id]/_components/entry-button.tsx`,
`front/app/games/[id]/_components/shot-list.tsx`,
`front/app/games/[id]/_components/shot-paths.ts`,
`front/app/games/[id]/_components/game-over-panel.tsx`, `front/app/games/[id]/page.tsx`,
`front/app/ui/global.css`

- `shot-paths.ts`: export `pairKey(a, b)`, the key `pathKey` builds today (the two entries
  sorted, joined by `-`), and make `pathKey` use it. A trace's key is `pairKey(entry, exit)`, so
  a fired shot's path and its trace share one key.
- `use-traces.ts`: from the revealed layout (absent while in progress → no traces), compute
  `traceBeam` for the 36 entries, deduplicated by key since both ends give the same trace. It
  exposes the traces, `keyAt(entry)`, the `selected` key (null at first) and `toggle(key)`:
  selecting another key replaces the selection, the same key clears it.
- `trace-lines.tsx`: an SVG `<g>` drawing traces as polylines in their beam colour, non-scaling
  stroke, round joins. A colour with an `outline` in `BEAM_SWATCHES` (`black`) gets that outline
  as a wider stroke underneath. A trace of colour `none` (it touched no piece) is never drawn,
  in either mode: selecting it shows nothing. The selected trace
  is animated drawing from its entry to its exit, under `motion-safe:` only, with a new
  `trace-draw` keyframe in `global.css` (pattern of `beam-a`: `pathLength=1`, dash offset), and
  restarts when the selection changes. Each trace carries `data-cy="trace"`, `data-entry`,
  `data-exit`, `data-colour`.
- `GridSvg` renders its `children` after the pieces; `GameBoard` passes the trace lines there
  when the layout is revealed.
- `EntryButton`: takes an optional `traceKey` and `onSelect(entry)`. With `onSelect` (a finished
  game), a click selects the trace instead of firing, the button is no longer `aria-disabled`,
  and hover/focus highlight `path?.key ?? traceKey`. While in progress nothing changes.
- `ShotList`: takes an optional `onSelect(key)`; when given, a click toggles that shot's trace
  (hover still only highlights).
- `page.tsx` is at 143 lines: move the finished-game column (board, status line, equivalence
  note, guest card) into `game-over-panel.tsx`, which owns `useTraces`, and pass the selection
  to `ShotList` through the page.

**Done when:** on a mocked finished game, clicking an entry whose shot touched a piece draws one trace from it to its exit in
the shot's final colour, one that touched nothing draws none, clicking it again removes it, clicking another entry (fired or not)
replaces it, clicking a shot in the list does the same, the in-progress board still fires on
click, and `tsc` and `oxlint` are clean.

### Task 3: Selected / All toggle

**Files:** `front/app/games/[id]/_viewmodels/use-traces.ts`,
`front/app/games/[id]/_viewmodels/use-trace-mode.ts`,
`front/app/games/[id]/_components/trace-mode-toggle.tsx`,
`front/app/games/[id]/_components/game-over-panel.tsx`,
`front/app/games/[id]/_components/trace-lines.tsx`, `front/app/i18n/locales/en.ts`,
`front/app/i18n/locales/fr.ts`

- `use-trace-mode.ts`: `'selected' | 'all'`, default `'selected'`, kept in `localStorage` under
  `scout-beam.trace-mode`, following `use-keyboard-layout.ts` (every access in try/catch).
- `trace-mode-toggle.tsx`: two joined `Button`s (`sm`, the active one `secondary`, the other
  `ghost`, `aria-pressed`), lucide `MousePointerClick` for Selected and `Layers` for All, in a
  group labelled by `t.game.traceModes`. Placed under the board in `game-over-panel.tsx`, on the
  line of the status message. `data-cy="trace-mode-selected"` and `data-cy="trace-mode-all"`.
- In All mode every deduplicated trace except those of colour `none` is drawn, opaque, without animation; the highlighted key
  (hovered entry or shot), else the selected one, is drawn last so it sits on top. Clicking still
  toggles the selection. Selected mode behaves as in Task 2.
- i18n keys, in `game`: `traceModes` ("Traces" / "Tracés"), `traceSelected` ("Selected" /
  "Sélection"), `traceAll` ("All" / "Tout").

**Done when:** on a mocked finished game, All draws one trace per distinct pair of entries whose beam touched a piece, the
hovered one comes on top, the choice survives a reload, the labels switch with the language, and
`tsc` and `oxlint` are clean; screenshots at 1366×700 and 390×844 show the toggle fitting under
the board.
