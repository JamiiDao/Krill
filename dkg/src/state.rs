#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum DkgState {
    Initial,
    Round1,
    Round2,
    Round3,
}
