//! Piece colours and the beam colour they mix into.

use serde::{Deserialize, Serialize};

/// The colour of a piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Colour {
    Red,
    Blue,
    Yellow,
    White,
}

/// The mix of the colours of the pieces a beam bounced off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeamColour {
    None,
    White,
    Red,
    Blue,
    Yellow,
    Violet,
    Orange,
    Green,
    Black,
}

const RED: u8 = 1;
const BLUE: u8 = 2;
const YELLOW: u8 = 4;
const WHITE: u8 = 8;

impl BeamColour {
    /// White only shows when nothing else was touched: it adds nothing to a mix.
    pub fn mix(bounces: impl IntoIterator<Item = Colour>) -> Self {
        let mask = bounces.into_iter().fold(0, |mask, colour| {
            mask | match colour {
                Colour::Red => RED,
                Colour::Blue => BLUE,
                Colour::Yellow => YELLOW,
                Colour::White => WHITE,
            }
        });

        match mask & (RED | BLUE | YELLOW) {
            0 if mask == 0 => Self::None,
            0 => Self::White,
            RED => Self::Red,
            BLUE => Self::Blue,
            YELLOW => Self::Yellow,
            3 => Self::Violet,
            5 => Self::Orange,
            6 => Self::Green,
            _ => Self::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

    use super::*;
    use Colour::{Blue, Red, White, Yellow};

    #[test]
    fn every_row_of_the_colour_table() {
        let table: [(&[Colour], BeamColour); 12] = [
            (&[], BeamColour::None),
            (&[White], BeamColour::White),
            (&[White, White], BeamColour::White),
            (&[Red], BeamColour::Red),
            (&[Blue], BeamColour::Blue),
            (&[Yellow], BeamColour::Yellow),
            (&[Red, Blue], BeamColour::Violet),
            (&[Red, Yellow], BeamColour::Orange),
            (&[Blue, Yellow], BeamColour::Green),
            (&[Red, Blue, Yellow], BeamColour::Black),
            (&[White, Red, Red], BeamColour::Red),
            (&[Yellow, White, Blue], BeamColour::Green),
        ];

        for (bounces, expected) in table {
            assert_eq!(
                BeamColour::mix(bounces.iter().copied()),
                expected,
                "{bounces:?}"
            );
        }
    }

    #[test]
    fn colours_serialize_in_snake_case() {
        assert_eq!(serde_json::to_string(&Yellow).unwrap(), "\"yellow\"");
        assert_eq!(
            serde_json::to_string(&BeamColour::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&BeamColour::Violet).unwrap(),
            "\"violet\""
        );
        assert_eq!(
            serde_json::from_str::<BeamColour>("\"black\"").unwrap(),
            BeamColour::Black
        );
    }
}
