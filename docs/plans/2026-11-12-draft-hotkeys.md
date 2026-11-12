# Draft hotkeys

**Goal:** pick the shape and colour of the next piece from the keyboard, in an AZERTY or QWERTY
layout the player chooses, and remove a placed piece with a right click or a long press.

## Global Constraints

- Work in `front/`, on branch `claude/peaceful-hawking-f5uty0`; follow `CLAUDE.md` (scout-beam).
- Check with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware <files>` from `front/`.
- Keep every existing `data-cy`; add one on anything new a test would target.
- No hardcoded string: `useI18n()`, keys in both `en.ts` and `fr.ts`. Files ≤150 lines.
- No `contexts/` folder: shared state lives in the page and its `_viewmodels/`.
- Hotkeys follow the rules `R` already follows in `use-draft.ts`: matched on `event.key`
  lower-cased, ignored while typing (`typing()`) and with Ctrl, Meta or Alt.
- To see the result: `pnpm dev` (port 3001), then the Playwright scripts in
  `/tmp/claude-0/-home-user/30df618a-a597-5246-987b-e2ba08279642/scratchpad/` (`shot-all.js`,
  `measure-task4.js`, run with `NODE_PATH=/opt/node22/lib/node_modules`), which mock the API.

---

### Task 1: AZERTY / QWERTY setting

**Files:** `front/app/lib/keyboard.ts` (new),
`front/app/games/[id]/_viewmodels/use-keyboard-layout.ts` (new),
`front/app/games/[id]/_components/keyboard-layout-toggle.tsx` (new),
`front/app/games/[id]/_components/game-header.tsx`, `front/app/games/[id]/page.tsx`,
`front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

`lib/keyboard.ts` exports the type `KeyboardLayout` (`'azerty' | 'qwerty'`) and `KEYMAPS`, the
keys of each layout, lower-case, same physical positions:

- `azerty`: `shapePrevious: 'q'`, `shapeNext: 'd'`, `colourPrevious: 'a'`, `colourNext: 'e'`
- `qwerty`: `shapePrevious: 'a'`, `shapeNext: 'd'`, `colourPrevious: 'q'`, `colourNext: 'e'`

The hook `useKeyboardLayout()` returns `{ keyboardLayout, toggle }`. Until the player picks one,
it follows the interface language (`fr` → AZERTY, `en` → QWERTY, from `useI18n().locale`);
once toggled, the choice is stored in localStorage under `scout-beam.keyboard-layout` and wins.
Read storage in an effect and guard it with try/catch, as `app/i18n/index.tsx` does for the locale.

`page.tsx` calls the hook once and passes `keyboardLayout` down (Task 2 will hand it to
`DraftPanel`) and `keyboardLayout` + `onToggleKeyboardLayout` to `GameHeader`. Never a bare
`layout`: in `CONTEXT.md` a Layout is the set of pieces. `KeyboardLayoutToggle` looks like `LanguageSwitcher`
(ghost, `sm`, lucide `Keyboard` icon + `AZERTY` or `QWERTY`), `data-cy='keyboard-layout'`,
`data-keyboard-layout`, and sits just before `LanguageSwitcher` in the header's right-hand group,
only while the game is in progress. i18n keys: `keyboard.switch` (aria-label and title:
`'Switch keyboard layout'` / `'Changer la disposition du clavier'`), `keyboard.azerty`
(`'AZERTY'`), `keyboard.qwerty` (`'QWERTY'`).

**Done when:** in French the button reads AZERTY, in English QWERTY; a click toggles it, and the
choice survives a reload and a language change; the game header still fits at 390px wide.

### Task 2: Shape and colour hotkeys

**Files:** `front/app/games/[id]/_viewmodels/use-draft.ts`,
`front/app/games/[id]/_components/draft-panel.tsx`,
`front/app/games/[id]/_components/draft-toolbar.tsx`, `front/app/games/[id]/page.tsx`,
`front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

`useDraft(gameId, keyboardLayout)` takes the `KeyboardLayout` from `@/app/lib/keyboard` (Task 1);
the page passes it to `DraftPanel` as a `keyboardLayout` prop, which hands it to `useDraft` and to `DraftToolbar`.
Extend the existing keydown listener with the `KEYMAPS[keyboardLayout]` keys:

- `shapePrevious` / `shapeNext`: move to the previous / next shape in `SHAPES` order, skipping
  placed ones, wrapping around; nothing when no shape is picked (all five placed). Picking a
  shape this way keeps the current rotation and colour, as a click on the toolbar does.
- `colourPrevious` / `colourNext`: previous / next in `PIECE_COLOURS`, wrapping; always active.

Tooltips (`title`) show the keys of the current layout, upper-cased: on each shape button,
`draft.shapeHint` = `'{{shape}} ({{previous}} / {{next}})'` (same in both locales), on each
colour button `draft.colourHint` = `'{{colour}} ({{previous}} / {{next}})'` (same). The
`aria-label`s stay as they are.

**Done when:** in AZERTY, Q/D walk the free shapes and A/E the colours, wrapping, with the toolbar
following; in QWERTY the same with A/D and Q/E; keys do nothing with Ctrl held or once all five
pieces are placed (shape keys only); tooltips show the right letters after a layout toggle.

### Task 3: Remove a piece with a right click or a long press

**Files:** `front/app/games/[id]/_components/draft-board.tsx`,
`front/app/games/[id]/_components/grid-svg.tsx`, `front/app/i18n/locales/en.ts`,
`front/app/i18n/locales/fr.ts`

Today a left click on a placed piece removes it (`draft.remove`, which picks it back up). Now:

- A right click (`contextmenu`) on a placed piece, on its SVG group or on a cell it covers,
  removes it the same way. The browser menu is suppressed on the whole draft grid, empty cells
  included; a right click on an empty cell does nothing else.
- A long press of about 500 ms on a placed piece, with a touch pointer, removes it too (iOS fires
  no `contextmenu`; on Android, make sure one press does not remove twice). Moving or lifting
  the finger before then cancels it.
- A click on a placed piece no longer does anything, whatever triggers it (mouse, Enter). Space
  on a focused cell it covers removes it, so the grid stays usable without a mouse: handle it on
  `keydown` and prevent its default, so the page does not scroll and no synthetic click follows.
  Enter on a covered cell does nothing.
- `GridSvg`'s `onPieceClick` prop becomes `onPieceRemove`, wired to `contextmenu` and the long
  press. Texts: `game.removePiece` becomes `'Right-click to remove the {{shape}} piece'` /
  `'Clic droit pour retirer la pièce « {{shape}} »'`; `draft.cellRemove` keeps its wording (it is
  the keyboard path).

**Done when:** a right click on a placed piece removes it and selects its shape, rotation and
colour, with no browser menu; a left click or Enter on it does nothing; Space on a focused
covered cell removes it without scrolling the page; with touch emulation a 500 ms press removes it once and a short tap does nothing.
