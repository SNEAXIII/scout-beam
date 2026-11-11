//! Tracing a shot: from an entry, through the layout, to the entry the beam leaves by.

use super::colour::{BeamColour, Colour};
use super::grid::{COLUMNS, Entry, Layout, Occupancy, ROWS};

/// A beam crosses each cell at most once per direction, so a longer trace is a bug.
const STEP_CAP: usize = 4 * COLUMNS as usize * ROWS as usize;

/// The answer to a beam fired from `entry`. No serde: the HTTP answer is the game domain's DTO.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shot {
    pub entry: Entry,
    pub exit: Entry,
    pub beam_colour: BeamColour,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TraceError {
    #[error("the beam from entry {} did not leave the grid within {} steps", .0.number(), .1)]
    StepCapExceeded(Entry, usize),
}

/// Fire a beam from `entry`. The layout is not validated: a partial one traces just as well.
pub fn trace(layout: &Layout, entry: Entry) -> Result<Shot, TraceError> {
    trace_with_cap(layout, entry, STEP_CAP)
}

fn trace_with_cap(layout: &Layout, entry: Entry, cap: usize) -> Result<Shot, TraceError> {
    let occupancy = Occupancy::of(layout);
    let (mut cell, mut direction) = entry.launch();
    let mut bounces: Vec<Colour> = Vec::new();

    for _ in 0..cap {
        if let Some((fill, colour)) = occupancy.at(cell) {
            bounces.push(colour);
            direction = fill.deflect(direction);
        }
        match cell.step(direction) {
            Some(next) => cell = next,
            None => {
                return Ok(Shot {
                    entry,
                    exit: Entry::leaving(cell, direction),
                    beam_colour: BeamColour::mix(bounces),
                });
            }
        }
    }
    Err(TraceError::StepCapExceeded(entry, cap))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::engine::piece::{Piece, Rotation, Shape};

    fn entry(number: u8) -> Entry {
        Entry::new(number).unwrap()
    }

    fn piece(shape: Shape, colour: Colour, x: u8, y: u8, rotation: Rotation) -> Piece {
        Piece {
            shape,
            colour,
            x,
            y,
            rotation,
        }
    }

    fn shoot(pieces: Vec<Piece>, from: u8) -> (u8, BeamColour) {
        let shot = trace(&Layout { pieces }, entry(from)).unwrap();
        assert_eq!(shot.entry, entry(from));
        (shot.exit.number(), shot.beam_colour)
    }

    #[test]
    fn on_an_empty_grid_every_beam_crosses_to_the_opposite_entry() {
        for from in 1..=36_u8 {
            let opposite = match from {
                1..=10 | 19..=28 => 29 - from,
                _ => 47 - from,
            };
            assert_eq!(
                shoot(vec![], from),
                (opposite, BeamColour::None),
                "entry {from}"
            );
        }
    }

    // Entries reaching cell (4,3) by travel direction: from the top 5, the right 14, the bottom 24,
    // the left 33. A beam leaving (4,3) northward reaches 5, east 14, south 24, west 33.

    #[test]
    fn a_full_cell_sends_every_beam_back_to_its_own_entry() {
        let square = piece(Shape::Square, Colour::Blue, 4, 3, Rotation::Deg0);
        for from in [5, 14, 24, 33] {
            assert_eq!(shoot(vec![square], from), (from, BeamColour::Blue));
        }
    }

    #[test]
    fn a_half_cell_bounces_each_direction_off_its_straight_or_slanted_side() {
        // (spike rotation → corner, entry, exit)
        let table = [
            // South-west: straight south and west, slanted towards north-east.
            (Rotation::Deg0, 5, 14),
            (Rotation::Deg0, 14, 5),
            (Rotation::Deg0, 24, 24),
            (Rotation::Deg0, 33, 33),
            // North-west.
            (Rotation::Deg90, 5, 5),
            (Rotation::Deg90, 14, 24),
            (Rotation::Deg90, 24, 14),
            (Rotation::Deg90, 33, 33),
            // North-east.
            (Rotation::Deg180, 5, 5),
            (Rotation::Deg180, 14, 14),
            (Rotation::Deg180, 24, 33),
            (Rotation::Deg180, 33, 24),
            // South-east.
            (Rotation::Deg270, 5, 33),
            (Rotation::Deg270, 14, 14),
            (Rotation::Deg270, 24, 24),
            (Rotation::Deg270, 33, 5),
        ];

        for (rotation, from, exit) in table {
            let spike = piece(Shape::Spike, Colour::Yellow, 4, 3, rotation);
            assert_eq!(
                shoot(vec![spike], from),
                (exit, BeamColour::Yellow),
                "{rotation:?} from {from}"
            );
        }
    }

    #[test]
    fn a_beam_bouncing_twice_comes_back_to_its_own_entry_with_both_colours() {
        // Down into the red spike, east to the blue square, back west and up the way it came.
        let spike = piece(Shape::Spike, Colour::Red, 4, 3, Rotation::Deg0);
        let square = piece(Shape::Square, Colour::Blue, 7, 3, Rotation::Deg0);
        assert_eq!(shoot(vec![spike, square], 5), (5, BeamColour::Violet));
    }

    #[test]
    fn a_beam_through_a_diamond_picks_up_white() {
        // Diamond at (4,3): entering its top-left cell from the left, the slanted side sends it north.
        let diamond = piece(Shape::Diamond, Colour::White, 4, 3, Rotation::Deg0);
        assert_eq!(shoot(vec![diamond], 33), (5, BeamColour::White));
    }

    #[test]
    fn a_beam_that_never_meets_a_piece_carries_no_colour() {
        let square = piece(Shape::Square, Colour::Red, 4, 3, Rotation::Deg0);
        assert_eq!(shoot(vec![square], 1), (28, BeamColour::None));
    }

    #[test]
    fn a_trace_past_the_step_cap_is_an_error() {
        let layout = Layout::default();
        assert_eq!(
            trace_with_cap(&layout, entry(1), 3),
            Err(TraceError::StepCapExceeded(entry(1), 3))
        );
    }
}
