# Onboarding

**Goal:** a newcomer understands how to play without anyone explaining: a guided Tutorial, clearer
rules, readable shot markers and a first-shot hint in real games.

## Global Constraints

- Front only (`front/`): no API change, no new dependency, never modify `components/ui/`.
- Vocabulary from `CONTEXT.md`: Tutorial, Shot, Trace, Return, Draft, Proposition, Spike, Corner,
  Diamond. Never "laser" in code or copy.
- Every string through `useI18n()`, keys in both `i18n/locales/en.ts` and `fr.ts`.
- Game colours live in `lib/colours.ts`; Tailwind semantic tokens otherwise.
- Animations through `motion-safe:` with keyframes in `app/ui/global.css`; with reduced motion the
  final frame stays readable.
- `data-cy` on every element a test would target; files ≤150 lines; comments ≤2 lines and rare.
- Storage reads and writes wrapped in try/catch, keys prefixed `scout-beam.` like `lib/draft.ts`.
- Check: `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware` in `front/`.
- Do not commit.

---

### Task 1: Shot markers for misses and bounces

**Files:** `front/app/lib/colours.ts`, `front/app/games/[id]/_components/beam-marker.tsx`

Already in the working tree; review it as is. A Return (`path.from === path.to`) keeps its marker
unchanged. A miss (colour `none`) is a black dot with a solid white border (`MISS_MARKER`). Any other
shot (bounced, left by another entry) keeps its fill with a 2px dashed border, white dashes over
black gaps (`BOUNCE_RING`). The marker carries `data-return`.

**Done when:** on a game page, the three kinds of shot are told apart at the entries and in the
shot list, and the checks pass.

### Task 2: "Fire first" hint in a real game

**Files:** `front/app/games/[id]/_components/draft-panel.tsx`, `front/app/i18n/locales/en.ts`,
`front/app/i18n/locales/fr.ts`

While the game has no shot (`board.paths.fired.size === 0`), the `draft-count` span is replaced by a
hint with `data-cy='fire-first'`, key `draft.fireFirst`: en
`Start by firing from a number around the grid`, fr
`Commencez par tirer depuis un numéro autour de la grille`. Placing pieces stays allowed; the hint
goes at the first shot.

**Done when:** a fresh game shows the hint instead of "0 / 5", and the counter comes back after one
shot.

### Task 3: Tap hint, ghost piece and a guided board

**Files:** `front/components/tap-hint.tsx`, `front/components/ghost-piece.tsx`,
`front/app/games/[id]/_components/game-board.tsx`, `front/app/ui/global.css`

- `TapHint`: an absolutely placed, `pointer-events-none` icon over its parent. It shows lucide
  `Pointer` (a finger) by default and `MousePointer2` under `pointer-fine:`, CSS only, like
  `hotkey-legend.tsx`. It moves onto the target, presses (a slight shrink plus a widening ripple) and
  loops. Under reduced motion it rests on the target. `data-cy='tap-hint'`.
- `GhostPiece`: SVG for inside a `GridSvg`. It draws a `Piece` with `pieceCells` and `cellPoints`
  as a translucent dashed outline in the piece's colour. An `eliminated` prop fades it and crosses it
  out with a transition. `data-cy='ghost-<shape>'` and `data-eliminated`.
- `GameBoard` gains an optional `guide`, either an entry number or a grid cell `{ x, y }`. With an
  entry, every other entry is disabled and a `TapHint` sits in that entry's ring cell. With a cell,
  the entries keep their behaviour and a `TapHint` sits over that grid cell (CSS grid placement
  `x + 2`, `y + 2`, as `ringCell` does). Without `guide`, nothing changes for the game page or the
  daily.

**Done when:** the existing pages behave as before, and a throwaway render of `GameBoard` with
`guide={3}` or `guide={{ x: 4, y: 2 }}` shows the hint at the right place (not committed).

### Task 4: Tutorial script and its check

**Files:** `front/app/tutorial/_lib/script.ts`, `front/app/tutorial/_lib/script.check.ts`,
`front/app/lib/trace.ts`, `front/tsconfig.json`, `front/package.json`

`script.ts` exports `TUTORIAL_LAYOUT` (a red Square and a yellow Corner on the 10×8 grid) and
`TUTORIAL_STEPS`, in this order:

1. A shot that touches nothing.
2. A Return off the Square's straight side (red).
3. A shot turned 90° by a slanted side of the Corner. It opens three candidates, a Spike, a Corner
   and a Diamond, each placed where it would give the same answer as the shots fired so far.
4. A shot that eliminates one candidate.
5. A shot that eliminates a second one, leaving the Corner.
6. A shot that touches both pieces (orange).
7. Draft: place both pieces.
8. Propose.

A shot step names its entry, and its `candidates` / `eliminated` lists where relevant. Answers are
not written in the script: they come from `traceBeam(TUTORIAL_LAYOUT, entry)`.

`script.check.ts` is a plain `node:assert` script, run with `node app/tutorial/_lib/script.check.ts`
(Node 24 strips types). It asserts:
- each step's expected kind: miss = colour `none`, Return = same exit, 90° turn, orange;
- each candidate, put in place of the Corner beside the Square, matches the hidden layout's answer
  on every shot before its elimination step and differs on that step;
- the Corner candidate is never eliminated.

Node resolves no extensionless or `@/` import, so the chain the check loads uses relative imports
with `.ts` extensions. Change `trace.ts`'s value import of `./pieces` to `./pieces.ts` and add
`"allowImportingTsExtensions": true` to `tsconfig.json`. Add the script
`"check:tutorial": "node app/tutorial/_lib/script.check.ts"` to `package.json`.

**Done when:** `pnpm check:tutorial` passes, and fails when one coordinate of the layout is moved.

### Task 5: Tutorial page

**Files:** `front/app/tutorial/page.tsx`, `front/app/tutorial/_viewmodels/use-tutorial.ts`,
`front/app/tutorial/_components/*`, `front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

`/tutorial` plays `TUTORIAL_STEPS` from `tutorial/_lib/script.ts` in the browser, with no API call.
`use-tutorial` holds the step, the shots fired (as `Shot`s built from `traceBeam`, fed to `shotPaths`)
and whether the current Trace is shown.

- **Shot steps (1–6):** `GameBoard` with an empty layout (`{ pieces: [] }`, so `GridSvg` draws its
  children) and `guide` on the step's entry. After the shot, the step's text explains the answer.
  Then a "See the path" button, labelled as tutorial-only, draws that shot's Trace with `TraceLines`.
  "Next" moves on. From step 3, the `GhostPiece` candidates are drawn and `eliminated` follows the
  script. Each step's text names the marker the player just got: a miss's black dot, a Return, a
  bounce's dashed ring (see Task 1).
- **Draft locked:** during steps 1–6 no draft is shown, and a line reads `tutorial.fireFirst`: en
  `Fire first: the pieces are deduced from the shots`, fr
  `Tirez d'abord : les pièces se déduisent des tirs`.
- **Step 7:** `DraftToolbar` plus `GameBoard` with `useDraft('tutorial', …)`. `guide` is on a cell of
  the next missing piece.
- **Step 8:** a propose button with a `TapHint`. The Proposition is correct when `traceBeam` of the
  draft equals `traceBeam(TUTORIAL_LAYOUT)` on all 36 entries (exit and colour). A wrong one says so
  and lets the player retry.
- **End:** `data-cy='tutorial-done'`, links to `/` (today's daily) and `/rules`. Sets storage key
  `scout-beam.tutorial` to `done` and clears the `tutorial` draft (`clearDraft`).
- Shown above the board: a step counter and a restart link that clears the draft and the step.

i18n under `tutorial.*`, one text per step. Write the copy from the decisions: the beam's path is
never shown in a real game, and a 90° turn can come from a Spike, a Corner or a Diamond.

**Done when:** a player can go through all eight steps on desktop and on a phone-width viewport,
only the guided entry can be fired in steps 1–6, and `scout-beam.tutorial` is `done` at the end.

### Task 6: Ways into the Tutorial

**Files:** `front/app/_components/tutorial-card.tsx`, `front/app/page.tsx`,
`front/app/rules/_components/landing-hero.tsx`, `front/app/i18n/locales/en.ts`,
`front/app/i18n/locales/fr.ts`

- Home: a card above the daily board, shown when `scout-beam.tutorial` (Task 5) is unset. It is read
  after mount so the prerender never shows it. Title `tutorial.cardTitle` (en
  `New here? Learn to play in 2 minutes`, fr `Nouveau ? Apprenez à jouer en 2 minutes`), a primary
  link to `/tutorial` (`data-cy='tutorial-start'`), and a "Later" button (`data-cy='tutorial-later'`)
  that stores `dismissed` and hides the card.
- `/rules` hero: a "Tutorial" button linking to `/tutorial` (`data-cy='rules-tutorial'`), beside
  "Play".

**Done when:** a fresh browser sees the card; after "Later" or a finished Tutorial it is gone on
reload; the rules hero links to `/tutorial`.

### Task 7: Rules — the shot and its invisible path

**Files:** `front/app/rules/_components/shot-demo.tsx`, `front/app/rules/_components/rules-section.tsx`,
`front/app/ui/global.css`, `front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

- `ShotDemo` illustrates the `shot` rule, on a small grid like `beam-demo.tsx`, with the ring's entry
  numbers drawn. A `TapHint` taps an entry, a beam runs to a different exit entry and picks up red,
  then its path fades out. Only both end markers remain (in `BeamMarker`'s style, with label A),
  next to the answer as the shot list shows it (`A · <entry> → <exit>`) and the caption "What you
  see". Loops.
- A new rule `landing.rules.invisible`, right after `shot`, set apart as a callout. en: `You never
  see the beam's path: only the entry it leaves by and its colour. Deducing the path is up to you; it
  is drawn only once the game is over.` Plus its French translation.
- If `rules-section.tsx` passes 150 lines, move `Rule` into its own file.

**Done when:** `/rules` shows the animation and the callout in both languages, and the animation's
final frame is readable under reduced motion.

### Task 8: Rules — sides demo and reading the grid

**Files:** `front/app/rules/_components/beam-demo.tsx`, `front/app/rules/_components/markers-demo.tsx`,
`front/app/rules/_components/rules-section.tsx`, `front/app/ui/global.css`,
`front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

- `BeamDemo`: the first example now leaves by a different entry than it was fired from. A second,
  labelled example keeps the beam that comes back out of its own entry, a Return. Update
  `landing.beamDemo` to describe both.
- New rule `landing.rules.markers` after `colours`, illustrated by `MarkersDemo`. It renders real
  `BeamMarker`s from hand-built `ShotPath`s: a miss, a bounce out of another entry (dashed ring) and
  a Return, each with a one-line caption. Its text explains that both ends of a shot show the same
  letter and colour, and that the shot list repeats them.

**Done when:** `/rules` shows both sides examples and the three markers with captions, in en and fr.

### Task 9: Rules — deducing

**Files:** `front/app/rules/_components/deduce-demo.tsx`, `front/app/rules/_components/rules-section.tsx`,
`front/app/ui/global.css`, `front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

New rule `landing.rules.deduce` after `markers`. en: `A turn does not say which piece caused it: a
Spike, a Corner or a Diamond can all turn a beam by 90°. Cross more shots through the same spot to
rule them out.` Plus its French translation.

`DeduceDemo` uses `GhostPiece` (Task 3) on a small grid. One shot turns, three ghost candidates
appear, a second shot crosses the area and one candidate is eliminated, a third leaves a single one.
Loops. The geometry is hand-drawn and must be truthful: each eliminated candidate would really have
sent that crossing shot elsewhere.

**Done when:** `/rules` shows the rule and its animation, in en and fr, readable under reduced motion.

---

## Round 2 — after the first playtest

Decisions: the tap hint must land on its target; its copy names no hand; the Tutorial opens on
the five pieces; each shot step plays its Trace on "Next" instead of a button; candidates show one
at a time, filled, over the outlines of the others; a "Previous" button.

### Task 10: Tap hint lands on its target

**Files:** `front/app/games/[id]/_components/game-board.tsx`, `front/app/i18n/locales/en.ts`,
`front/app/i18n/locales/fr.ts`

Today the hint is an absolutely positioned grid child given only a start line (`ringCell` returns
`gridColumn: 3`): for an absolute item the missing end line is the container's edge, so the hint
lands mid-board. Wrap `TapHint` in an in-flow grid item placed on the target's cell
(`pointer-events-none relative`, same `gridColumn`/`gridRow`), overlapping the entry or cell; the
hint stays `absolute inset-0` inside it. `tutorial.fire` names no hand: en
`Fire from entry {{entry}}: tap the number shown.`, fr
`Tirez depuis l’entrée {{entry}} : touchez le numéro indiqué.` Same for any other tutorial text
mentioning the hand (`step7`): "the cell shown" / « la case indiquée ».

**Done when:** with `guide={1}` the hint's tip sits on entry 1, with a cell guide on that cell.

### Task 11: The Tutorial opens on the five pieces

**Files:** `front/app/tutorial/_lib/script.ts`, `front/app/tutorial/_lib/script.check.ts`,
`front/app/tutorial/_viewmodels/use-tutorial.ts`, `front/app/tutorial/page.tsx`,
`front/app/tutorial/_components/tutorial-intro.tsx`, locales

A first step `{ kind: 'intro' }` in `TUTORIAL_STEPS`; the check accepts it first (intro, shot ×6,
draft, propose). Its view: a `GridSvg` with `INTRO_LAYOUT` (exported from `script.ts`), the five
shapes placed apart at fixed spots, each named under it (`t.shapes`), colours spread over red,
blue, yellow, white. Text `tutorial.steps.intro`: the goal (find where the pieces are hidden by
firing from the numbers), the five shapes, and "in a real game all five are hidden; here, to learn,
only two: a red Square and a yellow piece whose shape you will deduce". A "Start" button carries a
`TapHint`. Step texts are keyed by step, not by index arithmetic, so inserting a step shifts
nothing.

**Done when:** `/tutorial` opens on the five named pieces, "Start" leads to the first shot, the
counter counts nine steps, `pnpm check:tutorial` passes.

### Task 12: Shot steps in two beats, and "Previous"

**Files:** `front/app/tutorial/_viewmodels/use-tutorial.ts`,
`front/app/tutorial/_components/tutorial-shots.tsx`, `front/app/tutorial/page.tsx`, locales

Each shot step has three phases: `aim` (guided entry, `tutorial.fire`), `answer` (markers only;
text `tutorial.answer`: en `Answer: out by entry {{exit}}, {{colour}}. That is all you will see in
a real game.`, fr `Réponse : sortie par l’entrée {{exit}}, {{colour}}. C’est tout ce que vous verrez
en vraie partie.`; a miss says "no colour"), `explain` (the Trace animates by itself with
`TraceLines` `animated` and `misses`, text = the step's existing explanation). "Next" goes
answer → explain → next step; the `TapHint` sits on "Next" in `answer` and `explain`. Remove the
see-trace button and its keys.

"Previous" (`data-cy='tutorial-previous'`) beside "Next", disabled on the intro: explain → answer,
answer → aim (the step's shot removed), aim → the previous step's last phase (`explain` for a shot
step). From the draft steps it returns to step 6's `explain`; the draft is kept. Candidates and
eliminations follow the shots, so they roll back with them.

**Done when:** each shot shows the bare answer first, the Trace on "Next", and "Previous" walks
back through every phase to the intro with markers and ghosts matching each state.

### Task 13: Candidates one at a time

**Files:** `front/components/ghost-piece.tsx`, `front/app/tutorial/_components/tutorial-shots.tsx`,
`front/app/tutorial/_viewmodels/use-tutorial.ts` (or a small hook beside it)

All open candidates stay drawn as dashed outlines; one of the non-eliminated ones at a time is
`highlighted` — solid stroke, strong fill — switching every 1.5 s among the non-eliminated
(none switch under reduced motion: the first stays highlighted). A "?" badge sits on the turn's
cell while more than one candidate remains. Eliminated ones are crossed out and leave the rotation.
`GhostPiece` gains `highlighted?: boolean`; `/rules`' deduce demo is unchanged.

**Done when:** at step 3 the three shapes take turns being filled with a "?" on the spot; after
steps 4 and 5 the ruled-out ones are crossed and skipped; the Corner alone remains, without "?".
