/// Chess board representation and position management.
///
/// This module provides:
/// - `BoardBuilder`: Builder for constructing chess positions from FEN or FRC
/// - `CoreBoard`: Core board representation without move history
/// - `Board`: Board representation with move history tracking
/// - `StateInfo`: State information for board positions
/// - `Outcome`: Game outcome detection (checkmate, stalemate, draws)
///
/// # Examples
///
/// ```
/// use smolchess::board::BoardBuilder;
///
/// let core = BoardBuilder::classical().build().unwrap();
/// ```
mod board;
mod builder;
mod outcome;
pub(crate) mod zobrist;

#[cfg(feature = "alloc")]
pub use board::Board;
pub use board::CoreBoard;
pub use board::StateInfo;
pub use builder::BoardBuilder;
pub use outcome::Outcome;
