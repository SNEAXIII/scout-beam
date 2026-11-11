//! The five shapes, the cells they cover and how a cell turns a beam.

use serde::{Deserialize, Serialize};

use super::colour::Colour;
use super::grid::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    Square,
    Bar,
    Spike,
    Corner,
    Diamond,
}

/// The corner a half cell fills: its two straight sides are the grid sides meeting there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellCorner {
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellFill {
    Full,
    Half(CellCorner),
}

/// Quarter turns clockwise, carried as `0..=3` on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("a rotation is 0 to 3 quarter turns, got {0}")]
pub struct RotationError(pub u8);

/// A piece of a layout: a shape, its colour, its rotation, and the top-left of its bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub shape: Shape,
    pub colour: Colour,
    pub x: u8,
    pub y: u8,
    pub rotation: Rotation,
}

/// One cell a piece covers, in grid coordinates that may lie outside the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PieceCell {
    pub x: i16,
    pub y: i16,
    pub fill: CellFill,
}

impl Shape {
    pub const ALL: [Self; 5] = [
        Self::Square,
        Self::Bar,
        Self::Spike,
        Self::Corner,
        Self::Diamond,
    ];

    const fn base_cells(self) -> &'static [(i16, i16, CellFill)] {
        use CellCorner::{NorthEast, NorthWest, SouthEast, SouthWest};
        use CellFill::{Full, Half};

        match self {
            Self::Square => &[(0, 0, Full)],
            Self::Bar => &[(0, 0, Full), (1, 0, Full)],
            Self::Spike => &[(0, 0, Half(SouthWest))],
            Self::Corner => &[
                (0, 0, Half(SouthWest)),
                (0, 1, Full),
                (1, 1, Half(SouthWest)),
            ],
            Self::Diamond => &[
                (0, 0, Half(SouthEast)),
                (1, 0, Half(SouthWest)),
                (0, 1, Half(NorthEast)),
                (1, 1, Half(NorthWest)),
            ],
        }
    }
}

impl CellCorner {
    const fn rotated_clockwise(self) -> Self {
        match self {
            Self::NorthWest => Self::NorthEast,
            Self::NorthEast => Self::SouthEast,
            Self::SouthEast => Self::SouthWest,
            Self::SouthWest => Self::NorthWest,
        }
    }

    const fn straight_sides(self) -> (Direction, Direction) {
        match self {
            Self::NorthEast => (Direction::North, Direction::East),
            Self::NorthWest => (Direction::North, Direction::West),
            Self::SouthEast => (Direction::South, Direction::East),
            Self::SouthWest => (Direction::South, Direction::West),
        }
    }
}

impl CellFill {
    const fn rotated_clockwise(self) -> Self {
        match self {
            Self::Full => Self::Full,
            Self::Half(corner) => Self::Half(corner.rotated_clockwise()),
        }
    }

    /// The direction a beam travelling `travel` leaves this cell in.
    pub fn deflect(self, travel: Direction) -> Direction {
        let Self::Half(corner) = self else {
            return travel.reverse();
        };
        let (first, second) = corner.straight_sides();
        let entered_through = travel.reverse();

        if entered_through == first || entered_through == second {
            travel.reverse()
        } else if entered_through == first.reverse() {
            // The slanted side sends it out through the other empty side.
            second.reverse()
        } else {
            first.reverse()
        }
    }
}

impl Rotation {
    const fn quarter_turns(self) -> u8 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 1,
            Self::Deg180 => 2,
            Self::Deg270 => 3,
        }
    }
}

impl TryFrom<u8> for Rotation {
    type Error = RotationError;

    fn try_from(quarter_turns: u8) -> Result<Self, Self::Error> {
        match quarter_turns {
            0 => Ok(Self::Deg0),
            1 => Ok(Self::Deg90),
            2 => Ok(Self::Deg180),
            3 => Ok(Self::Deg270),
            other => Err(RotationError(other)),
        }
    }
}

impl From<Rotation> for u8 {
    fn from(rotation: Rotation) -> Self {
        rotation.quarter_turns()
    }
}

impl Piece {
    /// The cells this piece covers once rotated, its bounding box anchored at `(x, y)`.
    pub fn cells(&self) -> Vec<PieceCell> {
        let mut cells: Vec<PieceCell> = self
            .shape
            .base_cells()
            .iter()
            .map(|&(x, y, fill)| PieceCell { x, y, fill })
            .collect();

        for _ in 0..self.rotation.quarter_turns() {
            for cell in &mut cells {
                (cell.x, cell.y) = (-cell.y, cell.x);
                cell.fill = cell.fill.rotated_clockwise();
            }
        }

        let min_x = cells.iter().map(|cell| cell.x).min().unwrap_or(0);
        let min_y = cells.iter().map(|cell| cell.y).min().unwrap_or(0);
        for cell in &mut cells {
            cell.x += i16::from(self.x) - min_x;
            cell.y += i16::from(self.y) - min_y;
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use std::collections::HashSet;

    use super::CellCorner::{NorthEast, NorthWest, SouthEast, SouthWest};
    use super::CellFill::{Full, Half};
    use super::Direction::{East, North, South, West};
    use super::*;

    fn cells(shape: Shape, rotation: Rotation) -> HashSet<(i16, i16, CellFill)> {
        let piece = Piece {
            shape,
            colour: Colour::Red,
            x: 0,
            y: 0,
            rotation,
        };
        piece
            .cells()
            .into_iter()
            .map(|c| (c.x, c.y, c.fill))
            .collect()
    }

    #[test]
    fn a_full_cell_turns_back_every_direction() {
        for travel in [North, East, South, West] {
            assert_eq!(Full.deflect(travel), travel.reverse());
        }
    }

    #[test]
    fn a_half_cell_turns_back_on_its_straight_sides_and_by_90_on_its_slanted_side() {
        // (corner, travel, leaves towards)
        let table = [
            (NorthEast, South, North),
            (NorthEast, West, East),
            (NorthEast, North, West),
            (NorthEast, East, South),
            (NorthWest, South, North),
            (NorthWest, East, West),
            (NorthWest, North, East),
            (NorthWest, West, South),
            (SouthEast, North, South),
            (SouthEast, West, East),
            (SouthEast, South, West),
            (SouthEast, East, North),
            (SouthWest, North, South),
            (SouthWest, East, West),
            (SouthWest, South, East),
            (SouthWest, West, North),
        ];

        for (corner, travel, expected) in table {
            assert_eq!(
                Half(corner).deflect(travel),
                expected,
                "{corner:?} {travel:?}"
            );
        }
    }

    #[test]
    fn a_bar_turns_upright_a_quarter_turn_clockwise() {
        assert_eq!(
            cells(Shape::Bar, Rotation::Deg90),
            HashSet::from([(0, 0, Full), (0, 1, Full)])
        );
    }

    #[test]
    fn a_spike_corner_rotates_clockwise() {
        let expected = [
            (Rotation::Deg0, SouthWest),
            (Rotation::Deg90, NorthWest),
            (Rotation::Deg180, NorthEast),
            (Rotation::Deg270, SouthEast),
        ];
        for (rotation, corner) in expected {
            assert_eq!(
                cells(Shape::Spike, rotation),
                HashSet::from([(0, 0, Half(corner))])
            );
        }
    }

    #[test]
    fn a_corner_rotates_around_its_bounding_box() {
        assert_eq!(
            cells(Shape::Corner, Rotation::Deg90),
            HashSet::from([
                (1, 0, Half(NorthWest)),
                (0, 0, Full),
                (0, 1, Half(NorthWest))
            ])
        );
        assert_eq!(
            cells(Shape::Corner, Rotation::Deg180),
            HashSet::from([
                (1, 1, Half(NorthEast)),
                (1, 0, Full),
                (0, 0, Half(NorthEast))
            ])
        );
    }

    #[test]
    fn a_diamond_looks_the_same_at_every_rotation() {
        let upright = cells(Shape::Diamond, Rotation::Deg0);
        for rotation in [Rotation::Deg90, Rotation::Deg180, Rotation::Deg270] {
            assert_eq!(cells(Shape::Diamond, rotation), upright);
        }
    }

    #[test]
    fn cells_are_offset_by_the_piece_position() {
        let piece = Piece {
            shape: Shape::Bar,
            colour: Colour::Blue,
            x: 3,
            y: 5,
            rotation: Rotation::Deg180,
        };
        let placed: HashSet<_> = piece.cells().into_iter().map(|c| (c.x, c.y)).collect();
        assert_eq!(placed, HashSet::from([(3, 5), (4, 5)]));
    }

    #[test]
    fn a_rotation_past_three_quarter_turns_is_rejected() {
        assert_eq!(Rotation::try_from(4), Err(RotationError(4)));
        assert!(serde_json::from_str::<Rotation>("4").is_err());
    }

    #[test]
    fn a_piece_serializes_in_snake_case() {
        let piece = Piece {
            shape: Shape::Diamond,
            colour: Colour::White,
            x: 2,
            y: 7,
            rotation: Rotation::Deg270,
        };
        let json = serde_json::to_value(piece).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"shape": "diamond", "colour": "white", "x": 2, "y": 7, "rotation": 3})
        );
        assert_eq!(serde_json::from_value::<Piece>(json).unwrap(), piece);
    }
}
