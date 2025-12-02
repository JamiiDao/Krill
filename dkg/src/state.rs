use core::fmt;

use wincode::{SchemaRead, SchemaWrite};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, SchemaRead, SchemaWrite,
)]
pub enum FrostDkgState {
    #[default]
    Initial,
    Part1,
    Part2,
    Part3,
    Finalized,
}

impl fmt::Display for FrostDkgState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_value = match self {
            Self::Initial => "Initial",
            Self::Part1 => "Part 1",
            Self::Part2 => "Part 2",
            Self::Part3 => "Part 3",
            Self::Finalized => "Finalized",
        };

        write!(f, "{display_value}")
    }
}
