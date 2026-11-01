/// Chess move representation and move-making functionality.
///
/// This module provides:
/// - `mv`: Move representation (`Move`, `MoveKind`) and move validation
/// - `make`: Move making and undoing on the board
/// - `movegen`: Move generation for finding all legal moves
///
/// # Examples
///
/// ```
/// use smolchess::{Square, mv::{Move, MoveKind}, board::BoardBuilder};
///
/// let mut board = BoardBuilder::classical().build().unwrap();
/// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
/// board.make_move(mv).unwrap();
/// ```
pub mod display;
pub mod make;
pub mod movegen;
pub mod mv;

pub use display::San;
pub use make::PieceChangesBuffer;
pub use movegen::MoveList;
pub use mv::{Move, MoveKind};
