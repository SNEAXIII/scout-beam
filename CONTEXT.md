# Scout Beam

A deduction game: a bot or an opponent hides pieces on a grid, the player fires beams from the
edges and deduces the hidden layout from where each beam leaves and which colour it picked up.

## Language

**Grid**:
The 10-column by 8-row board on which pieces are hidden.
_Avoid_: board, map

**Entry**:
One of the 36 numbered positions around the grid where a beam goes in or comes out.
_Avoid_: port, slot, edge number

**Piece**:
One of the five hidden shapes — Square, Bar, Spike, Corner, Diamond — each with a colour.
_Avoid_: gem, crystal, block

**Straight side**:
A piece side parallel to the grid lines; a beam that hits it turns back (180°).

**Slanted side**:
A piece side at 45°; a beam that hits it turns by 90°.
_Avoid_: mirror, diagonal

**Layout**:
The set of five pieces with their position, rotation and colour.
_Avoid_: configuration, solution, setup

**Bot**:
The server-side opponent that draws the hidden layout of a game.
_Avoid_: AI, hider

**Game**:
One attempt by a player to find one hidden layout.
_Avoid_: round, match, party

**Bot game**:
A game against the bot outside the daily, as many as the player wants, with surrender allowed;
every entry is fired from the start.
_Avoid_: practice, solo game

**Shot**:
A beam fired by the player from one entry, answered with an exit entry and a beam colour.
_Avoid_: wave, ray, probe

**Return**:
A shot that leaves by the entry it was fired from; by reversibility, it met a straight side.
_Avoid_: bounce, echo, reflection

**Trace**:
The route a beam follows through the revealed layout, from its entry to the entry it leaves by.
_Avoid_: laser, path, ray

**Tutorial**:
A guided practice game played in the browser on a fixed layout of two pieces, step by step; no
score, no ranking, nothing stored on the server.
_Avoid_: onboarding, walkthrough, demo

**Beam colour**:
The mix of the colours of the pieces a beam bounced off: none if it touched nothing, white if it
touched only white pieces.

**Draft**:
The pieces the player places on the grid, where they believe the hidden ones are.
_Avoid_: notes, sketch

**Proposition**:
A draft submitted as the answer; it is correct when it gives the same shot result as the hidden
layout on all 36 entries.
_Avoid_: guess, solution, answer

**Guest**:
A player without credentials, who may only play the daily; signing up turns the guest into that
account and keeps its games, signing in to an existing account moves to it the guest's dailies of
days that account has not played.
_Avoid_: anonymous, visitor

**Daily**:
The game of the day: one layout drawn by the bot for every player, one attempt each, no surrender;
every entry is fired from the start.
_Avoid_: puzzle of the day, challenge

**Score**:
The time a game against the bot took to end, plus one minute per wrong proposition, plus five
minutes if it ended in a surrender; lower is better. A daily's score ranks the players of that day.

**Points**:
What a finished game against the bot earns, decreasing logarithmically with its score; the global
ranking sums only those of dailies, solved on their day or later.

**Race**:
Two players hunting the same layout drawn by the bot; the first correct proposition wins.

**Duel**:
Two players each hiding a layout for the other to find; the first correct proposition wins.

**Shot allowance**:
In a Race or a Duel, the shots a player may fire right now: 15 when the game starts, then one more
earned after 5 s, the next 5.5 s later, each 0.5 s longer than the last, kept until fired.
_Avoid_: ammo, charges, priority

**Proposition cooldown**:
In a Race or a Duel, the wait after a proposition before the next one: 5 s, one more second per
wrong proposition, up to 10 s.

**Room**:
Where two players meet before a game against each other, identified by a 6-character code that is
also its invitation link.
_Avoid_: lobby, lounge, session

**Surrender**:
The player ending a game without solving it: the hidden layout is revealed and no more shots can be
fired.
_Avoid_: forfeit, give up
