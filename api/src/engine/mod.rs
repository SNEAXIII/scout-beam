//! The beam engine: given a layout and an entry, where the beam leaves and which colour it carries.
//!
//! Pure game rules — no HTTP, no database.

pub mod beam;
pub mod colour;
pub mod grid;
pub mod piece;

pub use beam::{Shot, TraceError, trace};
pub use colour::{BeamColour, Colour};
pub use grid::{COLUMNS, Entry, EntryError, Layout, LayoutError, ROWS};
pub use piece::{CellCorner, CellFill, Piece, Rotation, RotationError, Shape};
