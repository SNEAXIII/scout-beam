//! The grid, its numbered entries, and the layout of pieces hidden on it.

use serde::{Deserialize, Serialize};

use super::colour::Colour;
use super::piece::{CellFill, Piece, PieceCell, Shape};

pub const COLUMNS: u8 = 10;
pub const ROWS: u8 = 8;
const ENTRY_COUNT: u8 = 2 * (COLUMNS + ROWS);

/// A direction of travel; `y` grows downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    #[must_use]
    pub const fn reverse(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
}

/// A cell inside the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    x: u8,
    y: u8,
}

impl Cell {
    fn at(x: i16, y: i16) -> Option<Self> {
        let x = u8::try_from(x).ok().filter(|&x| x < COLUMNS)?;
        let y = u8::try_from(y).ok().filter(|&y| y < ROWS)?;
        Some(Self { x, y })
    }

    /// The neighbouring cell towards `direction`, or `None` past the edge.
    pub fn step(self, direction: Direction) -> Option<Self> {
        let (x, y) = (i16::from(self.x), i16::from(self.y));
        match direction {
            Direction::North => Self::at(x, y - 1),
            Direction::East => Self::at(x + 1, y),
            Direction::South => Self::at(x, y + 1),
            Direction::West => Self::at(x - 1, y),
        }
    }
}

/// A numbered position around the grid: top `1–10` left to right, right `11–18` top to bottom,
/// bottom `19–28` right to left, left `29–36` bottom to top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Entry(u8);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("an entry is numbered 1 to 36, got {0}")]
pub struct EntryError(pub u8);

impl Entry {
    pub fn new(number: u8) -> Result<Self, EntryError> {
        if (1..=ENTRY_COUNT).contains(&number) {
            Ok(Self(number))
        } else {
            Err(EntryError(number))
        }
    }

    pub fn all() -> impl Iterator<Item = Self> {
        (1..=ENTRY_COUNT).map(Self)
    }

    pub const fn number(self) -> u8 {
        self.0
    }

    /// The first cell a beam fired from here crosses, and its direction, perpendicular to the edge.
    pub const fn launch(self) -> (Cell, Direction) {
        let n = self.0;
        match n {
            1..=10 => (Cell { x: n - 1, y: 0 }, Direction::South),
            11..=18 => (
                Cell {
                    x: COLUMNS - 1,
                    y: n - 11,
                },
                Direction::West,
            ),
            19..=28 => (
                Cell {
                    x: 28 - n,
                    y: ROWS - 1,
                },
                Direction::North,
            ),
            _ => (Cell { x: 0, y: 36 - n }, Direction::East),
        }
    }

    /// The entry a beam reaches when it leaves `cell`, an edge cell, travelling `direction`.
    pub const fn leaving(cell: Cell, direction: Direction) -> Self {
        Self(match direction {
            Direction::North => cell.x + 1,
            Direction::East => 11 + cell.y,
            Direction::South => 28 - cell.x,
            Direction::West => 36 - cell.y,
        })
    }
}

impl TryFrom<u8> for Entry {
    type Error = EntryError;

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        Self::new(number)
    }
}

impl From<Entry> for u8 {
    fn from(entry: Entry) -> Self {
        entry.0
    }
}

/// The pieces hidden on the grid.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Layout {
    pub pieces: Vec<Piece>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LayoutError {
    #[error("the layout has no {0:?}")]
    MissingShape(Shape),
    #[error("the layout has more than one {0:?}")]
    DuplicateShape(Shape),
    #[error("the {0:?} sticks out of the grid")]
    OutsideGrid(Shape),
    #[error("the {0:?} and the {1:?} overlap")]
    Overlap(Shape, Shape),
    #[error("the {0:?} and the {1:?} share a cell side")]
    SharedSide(Shape, Shape),
}

impl Layout {
    /// Valid when it holds the five shapes once each, inside the grid, apart from one another.
    pub fn validate(&self) -> Result<(), LayoutError> {
        for shape in Shape::ALL {
            match self
                .pieces
                .iter()
                .filter(|piece| piece.shape == shape)
                .count()
            {
                0 => return Err(LayoutError::MissingShape(shape)),
                1 => {}
                _ => return Err(LayoutError::DuplicateShape(shape)),
            }
        }

        let placed: Vec<(Shape, Vec<PieceCell>)> = self
            .pieces
            .iter()
            .map(|piece| (piece.shape, piece.cells()))
            .collect();

        for (shape, cells) in &placed {
            if cells.iter().any(|cell| Cell::at(cell.x, cell.y).is_none()) {
                return Err(LayoutError::OutsideGrid(*shape));
            }
        }

        for (index, (shape, cells)) in placed.iter().enumerate() {
            for (other_shape, other_cells) in placed.iter().skip(index + 1) {
                let closest = cells
                    .iter()
                    .flat_map(|a| other_cells.iter().map(move |b| (a, b)))
                    .map(|(a, b)| (a.x - b.x).abs() + (a.y - b.y).abs())
                    .min();
                match closest {
                    Some(0) => return Err(LayoutError::Overlap(*shape, *other_shape)),
                    Some(1) => return Err(LayoutError::SharedSide(*shape, *other_shape)),
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

/// What covers each cell of the grid; cells outside it are dropped, the first piece wins a clash.
pub struct Occupancy {
    cells: [[Option<(CellFill, Colour)>; COLUMNS as usize]; ROWS as usize],
}

impl Occupancy {
    pub fn of(layout: &Layout) -> Self {
        let mut occupancy = Self {
            cells: [[None; COLUMNS as usize]; ROWS as usize],
        };
        for piece in &layout.pieces {
            for cell in piece.cells() {
                if let Some(Cell { x, y }) = Cell::at(cell.x, cell.y)
                    && let Some(slot) = occupancy
                        .cells
                        .get_mut(usize::from(y))
                        .and_then(|row| row.get_mut(usize::from(x)))
                {
                    slot.get_or_insert((cell.fill, piece.colour));
                }
            }
        }
        occupancy
    }

    pub fn at(&self, cell: Cell) -> Option<(CellFill, Colour)> {
        self.cells
            .get(usize::from(cell.y))
            .and_then(|row| row.get(usize::from(cell.x)))
            .copied()
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use crate::engine::piece::Rotation;

    fn piece(shape: Shape, x: u8, y: u8, rotation: Rotation) -> Piece {
        Piece {
            shape,
            colour: Colour::Red,
            x,
            y,
            rotation,
        }
    }

    fn valid_pieces() -> Vec<Piece> {
        vec![
            piece(Shape::Square, 0, 0, Rotation::Deg0),
            piece(Shape::Bar, 2, 0, Rotation::Deg0),
            piece(Shape::Spike, 5, 0, Rotation::Deg0),
            piece(Shape::Corner, 7, 0, Rotation::Deg0),
            piece(Shape::Diamond, 0, 3, Rotation::Deg0),
        ]
    }

    fn with(shape: Shape, replacement: Piece) -> Layout {
        let mut pieces = valid_pieces();
        let slot = pieces.iter_mut().find(|p| p.shape == shape).unwrap();
        *slot = replacement;
        Layout { pieces }
    }

    #[test]
    fn entry_numbers_outside_1_to_36_are_rejected() {
        assert_eq!(Entry::new(0), Err(EntryError(0)));
        assert_eq!(Entry::new(37), Err(EntryError(37)));
        assert_eq!(Entry::new(36).map(Entry::number), Ok(36));
        assert!(serde_json::from_str::<Entry>("37").is_err());
        assert_eq!(
            serde_json::to_string(&Entry::new(12).unwrap()).unwrap(),
            "12"
        );
    }

    #[test]
    fn each_edge_launches_inward_from_its_imposed_numbering() {
        let cases = [
            (1, 0, 0, Direction::South),
            (10, 9, 0, Direction::South),
            (11, 9, 0, Direction::West),
            (18, 9, 7, Direction::West),
            (19, 9, 7, Direction::North),
            (28, 0, 7, Direction::North),
            (29, 0, 7, Direction::East),
            (36, 0, 0, Direction::East),
        ];
        for (number, x, y, direction) in cases {
            let entry = Entry::new(number).unwrap();
            assert_eq!(entry.launch(), (Cell { x, y }, direction), "entry {number}");
        }
    }

    #[test]
    fn leaving_back_out_of_the_launch_cell_reaches_the_same_entry() {
        assert_eq!(Entry::all().count(), 36);
        for entry in Entry::all() {
            let (cell, direction) = entry.launch();
            assert_eq!(cell.step(direction.reverse()), None);
            assert_eq!(Entry::leaving(cell, direction.reverse()), entry);
        }
    }

    #[test]
    fn a_layout_with_the_five_shapes_apart_is_valid() {
        let layout = Layout {
            pieces: valid_pieces(),
        };
        assert_eq!(layout.validate(), Ok(()));
    }

    #[test]
    fn pieces_touching_only_by_a_corner_are_valid() {
        // Square at (2,2) meets the Diamond's (1,3) only diagonally.
        let layout = with(Shape::Square, piece(Shape::Square, 2, 2, Rotation::Deg0));
        assert_eq!(layout.validate(), Ok(()));
    }

    #[test]
    fn a_missing_shape_is_rejected() {
        let mut pieces = valid_pieces();
        pieces.retain(|p| p.shape != Shape::Spike);
        assert_eq!(
            Layout { pieces }.validate(),
            Err(LayoutError::MissingShape(Shape::Spike))
        );
    }

    #[test]
    fn a_duplicated_shape_is_rejected() {
        let mut pieces = valid_pieces();
        pieces.push(piece(Shape::Square, 9, 7, Rotation::Deg0));
        assert_eq!(
            Layout { pieces }.validate(),
            Err(LayoutError::DuplicateShape(Shape::Square))
        );
    }

    #[test]
    fn a_piece_outside_the_grid_is_rejected() {
        let layout = with(Shape::Bar, piece(Shape::Bar, 9, 6, Rotation::Deg0));
        assert_eq!(layout.validate(), Err(LayoutError::OutsideGrid(Shape::Bar)));

        let layout = with(Shape::Bar, piece(Shape::Bar, 9, 7, Rotation::Deg90));
        assert_eq!(layout.validate(), Err(LayoutError::OutsideGrid(Shape::Bar)));
    }

    #[test]
    fn overlapping_pieces_are_rejected() {
        let layout = with(Shape::Spike, piece(Shape::Spike, 3, 0, Rotation::Deg0));
        assert_eq!(
            layout.validate(),
            Err(LayoutError::Overlap(Shape::Bar, Shape::Spike))
        );
    }

    #[test]
    fn pieces_sharing_a_cell_side_are_rejected() {
        let layout = with(Shape::Spike, piece(Shape::Spike, 4, 0, Rotation::Deg0));
        assert_eq!(
            layout.validate(),
            Err(LayoutError::SharedSide(Shape::Bar, Shape::Spike))
        );
    }

    #[test]
    fn occupancy_keeps_the_cells_inside_the_grid() {
        let layout = Layout {
            pieces: vec![piece(Shape::Bar, 9, 7, Rotation::Deg0)],
        };
        let occupancy = Occupancy::of(&layout);
        assert_eq!(
            occupancy.at(Cell { x: 9, y: 7 }),
            Some((CellFill::Full, Colour::Red))
        );
        assert_eq!(occupancy.at(Cell { x: 8, y: 7 }), None);
    }
}
