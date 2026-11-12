# Compact game page

**Goal:** finish compacting the game page: no empty top row, a toolbar aligned with the board, a
shot list that scrolls instead of stretching the page, and "Abandonner" as the French label.

## Global Constraints

- Work in `front/`, on branch `claude/peaceful-hawking-f5uty0`; follow `CLAUDE.md` (scout-beam).
- Check with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware <files>` from `front/`.
- Keep every existing `data-cy`; never edit `components/ui/` (except `button.tsx`, `card.tsx`).
- No hardcoded string: `useI18n()`, keys in both `en.ts` and `fr.ts`. Files ≤150 lines.
- To see the result: `pnpm dev` (port 3001), then the Playwright script
  `/tmp/claude-0/-home-user/30df618a-a597-5246-987b-e2ba08279642/scratchpad/shot.js <out-dir>`,
  which mocks the API and takes screenshots at 1190×880, 1366×700 and 390×844. Edit its mocked
  `shots` if a task needs more of them.

---

### Task 1: Language switcher in each page header

**Files:** `front/app/layout.tsx`, `front/app/_components/home-header.tsx`,
`front/app/games/[id]/_components/game-header.tsx`, `front/app/login/page.tsx`

Remove the `LanguageSwitcher` row from the root layout. Render the existing
`@/components/language-switcher` unchanged (ghost, `sm`, icon + locale):

- home: in the right-hand group of `HomeHeader`, after the sign-out button;
- game: last item of `GameHeader`, after the surrender button. The surrender button keeps
  `ml-auto`, so the switcher still sits at the far right once the game is over and that button
  is gone (move `ml-auto` to the switcher in that case, or wrap both in one right-aligned group).
  The header may wrap on a phone, never overflow horizontally;
- login: top-right corner of the page, over the centered card (`main` made `relative`, the
  switcher absolutely positioned roughly where the old row put it).

**Done when:** the three pages show the switcher in those places, no page has the empty row
above its content, and the game header fits at 390px wide without horizontal scroll.

### Task 2: "Abandonner" as the French surrender label

**Files:** `front/app/i18n/locales/fr.ts`

Set `game.surrender` to `'Abandonner'`. English stays `'Surrender'`; every other French surrender
string already says "Abandonner" / "Abandonnée" / "Abandon" and stays.

**Done when:** in French, the header button and the confirmation dialog's confirm button read
"Abandonner".

### Task 3: Board column at the board's width

**Files:** `front/app/games/[id]/_components/game-board.tsx`,
`front/app/games/[id]/_components/draft-panel.tsx`, `front/app/games/[id]/page.tsx`

Today the viewport-height cap (`maxWidth` with `100dvh - 13rem`) sits on the board alone, so on a
short screen the board is centered while the toolbar and the "Proposer" row stay left-aligned.
Move that cap to the left column itself: export it from `game-board.tsx` (one style or constant,
built from the existing `RATIO`) and apply it, with `mx-auto w-full`, to the `DraftPanel` root and
to the revealed-board column in `page.tsx` (board + end-of-game message). The board then just
fills its column. The shot list stays in its own right-hand column, unchanged.

The old row above the page is gone after Task 1: recompute the `13rem` offset so that at
1366×700 the header, toolbar, board and "Proposer" row all fit without a page scroll, with no
more than a few rem left below.

**Done when:** at 1366×700 the toolbar's left edge, the board's left edge and the "Proposer"
button's left edge line up, the block is centered in its column, and nothing scrolls; at
1190×880 and 390×844 the layout is unchanged apart from that.

### Task 4: Scrolling shot list

**Files:** `front/app/games/[id]/_components/shot-list.tsx`, `front/app/games/[id]/page.tsx`

The "Tirs" card keeps its title fixed and scrolls only its list (`CardContent` already has
`min-h-0 overflow-y-auto` inside a flex column).

- From `md` up: the card grows with its shots up to the left column's height (level with the
  "Proposer" row, or with the end-of-game message), then its list scrolls; it never makes the grid
  row taller. The page grid uses `items-start` today; give the right-hand column a wrapper that
  takes the row's height without contributing to it (stretched, zero height, full min-height) and
  let the card inside be at most that tall.
- Below `md` (card stacked under the board): cap it at `max-h-64`.
- Few shots, or none (`shots-empty`): the card keeps its natural, short height on both.

**Done when:** with 25 mocked shots, at 1190×880 and 1366×700 the card's bottom lines up with the
"Proposer" row and its list scrolls while "Tirs" stays visible; at 390×844 the card is at most
16rem tall and scrolls; the page itself never gets taller because of the shots.
