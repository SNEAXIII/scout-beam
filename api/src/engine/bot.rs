//! The bot: draws a random valid layout for a new game.

use rand::Rng;
use rand::seq::SliceRandom;

use super::colour::Colour;
use super::grid::{COLUMNS, Layout, ROWS};
use super::piece::{Piece, Rotation, Shape};

/// About one random draw in nine is valid, so failing 10,000 in a row is odds of ~e^-1200:
/// the cap only bounds the loop without a panic.
const MAX_ATTEMPTS: u32 = 10_000;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("no valid layout drawn in {MAX_ATTEMPTS} attempts")]
pub struct DrawError;

/// Draws the five shapes at random positions and rotations until `Layout::validate` accepts them.
pub fn draw_layout(rng: &mut impl Rng) -> Result<Layout, DrawError> {
    let colours = draw_colours(rng);
    for _ in 0..MAX_ATTEMPTS {
        let pieces = Shape::ALL
            .into_iter()
            .zip(colours)
            .map(|(shape, colour)| Piece {
                shape,
                colour,
                x: rng.random_range(0..COLUMNS),
                y: rng.random_range(0..ROWS),
                rotation: draw_rotation(rng),
            })
            .collect();
        let layout = Layout { pieces };
        if layout.validate().is_ok() {
            return Ok(layout);
        }
    }
    Err(DrawError)
}

/// One colour per shape: red, blue and yellow once each, two more of any colour, shuffled.
fn draw_colours(rng: &mut impl Rng) -> [Colour; 5] {
    let mut colours = [
        Colour::Red,
        Colour::Blue,
        Colour::Yellow,
        draw_colour(rng),
        draw_colour(rng),
    ];
    colours.shuffle(rng);
    colours
}

fn draw_colour(rng: &mut impl Rng) -> Colour {
    match rng.random_range(0..4u8) {
        0 => Colour::Red,
        1 => Colour::Blue,
        2 => Colour::Yellow,
        _ => Colour::White,
    }
}

fn draw_rotation(rng: &mut impl Rng) -> Rotation {
    match rng.random_range(0..4u8) {
        0 => Rotation::Deg0,
        1 => Rotation::Deg90,
        2 => Rotation::Deg180,
        _ => Rotation::Deg270,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::*;

    #[test]
    fn a_thousand_seeded_layouts_are_valid_and_hold_the_three_primaries() {
        for seed in 0..1_000 {
            let layout = draw_layout(&mut StdRng::seed_from_u64(seed)).unwrap();
            assert_eq!(layout.validate(), Ok(()), "seed {seed}");
            for primary in [Colour::Red, Colour::Blue, Colour::Yellow] {
                assert!(
                    layout.pieces.iter().any(|piece| piece.colour == primary),
                    "seed {seed} has no {primary:?}"
                );
            }
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_layout() {
        let first = draw_layout(&mut StdRng::seed_from_u64(42)).unwrap();
        let second = draw_layout(&mut StdRng::seed_from_u64(42)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_give_different_layouts() {
        let first = draw_layout(&mut StdRng::seed_from_u64(1)).unwrap();
        let second = draw_layout(&mut StdRng::seed_from_u64(2)).unwrap();
        assert_ne!(first, second);
    }
}
