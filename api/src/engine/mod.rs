//! The beam engine: given a layout and an entry, where the beam leaves and which colour it carries.
//!
//! Pure game rules — no HTTP, no database.

pub mod beam;
pub mod bot;
pub mod colour;
pub mod grid;
pub mod piece;

pub use beam::{TraceError, same_shots, trace};
pub use bot::{DrawError, draw_layout};
pub use colour::BeamColour;
pub use grid::{Entry, Layout, LayoutError};
pub use piece::{Piece, Shape};
