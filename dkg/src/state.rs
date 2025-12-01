use wincode::{SchemaRead, SchemaWrite};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, SchemaRead, SchemaWrite,
)]
pub enum FrostDkgState {
    #[default]
    Initial,
    Part1,
    Part2,
    Finalized,
}
