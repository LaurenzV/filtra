/// How an input is extended beyond its borders (`edgeMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeMode {
    /// Repeat the color values at the nearest edge.
    Duplicate,
    /// Take the color values from the opposite edge.
    Wrap,
    /// Take the color values mirrored across the edge.
    Mirror,
    /// Extend with transparent black.
    None,
}
