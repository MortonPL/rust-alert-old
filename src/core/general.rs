/// Supported C&C games.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEnum {
    /// Command & Conquer, also called Tiberian Dawn or C&C1.
    TD,
    /// Red Alert, also called Red Alert 1 or C&C2 (in Germany).
    RA,
    /// Tiberian Sun, also called C&C3 (in Germany).
    TS,
    /// Firestorm, expansion of Tiberian Sun.
    FS,
    /// Red Alert 2.
    RA2,
    /// Yuri's Revenge, expansion of Red Alert 2.
    YR,
}
