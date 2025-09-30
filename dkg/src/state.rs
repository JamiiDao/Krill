use wincode::{SchemaRead, SchemaWrite};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default, SchemaRead, SchemaWrite,
)]
pub enum DkgState {
    #[default]
    Initial,
    Round1,
    Round2,
    Finalized,
}
