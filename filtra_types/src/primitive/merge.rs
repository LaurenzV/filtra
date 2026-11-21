/// Stacks its inputs with source-over, the first input at the bottom (`feMerge`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Merge;
