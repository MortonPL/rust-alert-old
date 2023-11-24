//! General data definitions used by other modules.

/// Supported C&C games.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum GameEnum {
    /// Command & Conquer, also called Tiberian Dawn or C&C1. Old MIX format, old CRC algo.
    TD,
    /// Red Alert, also called Red Alert 1 or C&C2 (in Germany). Old/New MIX format, old CRC algo.
    RA,
    /// Tiberian Sun, also called C&C3 (in Germany). New MIX format, new CRC algo.
    TS,
    /// Firestorm, expansion of Tiberian Sun. New MIX format, new CRC algo.
    FS,
    /// Red Alert 2. New MIX format, new CRC algo.
    RA2,
    /// Yuri's Revenge, expansion of Red Alert 2. New MIX format, new CRC algo.
    #[default]
    YR,
}
