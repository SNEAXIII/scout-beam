# Scout Beam

A solo deduction game: a bot hides pieces on a grid, the player fires beams from the edges and
deduces the hidden layout from where each beam leaves and which colour it picked up.

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

**Shot**:
A beam fired by the player from one entry, answered with an exit entry and a beam colour.
_Avoid_: wave, ray, probe

**Beam colour**:
The mix of the colours of the pieces a beam bounced off: none if it touched nothing, white if it
touched only white pieces.

**Draft**:
The player's own grid where they place the pieces they believe are hidden.
_Avoid_: notes, sketch

**Proposition**:
A draft submitted as the answer; it is correct when it gives the same shot result as the hidden
layout on all 36 entries.
_Avoid_: guess, solution, answer
