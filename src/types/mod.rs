//! Fundamental types for representing chess concepts.
//!
//! This module provides the core data types used throughout the chess library:
//!
//! - **Color**: Represents the two sides in chess (White and Black)
//! - **PieceKind**: Represents the six types of chess pieces (Pawn, Knight, Bishop, Rook, Queen, King)
//! - **Piece**: Represents a complete chess piece with both color and kind
//! - **File**: Represents the columns of a chess board (A through H)
//! - **Rank**: Represents the rows of a chess board (1 through 8)
//! - **Square**: Represents a specific square on the chess board (A1 through H8)
//! - **CastlingSide**: Represents which side castling can occur on (Kingside or Queenside)
//! - **CastlingRights**: Tracks which castling rights are available for both players
//!
//! All types are designed to be efficient with `repr(u8)` for compact representation
//! and provide const-friendly methods for compile-time computations.

mod castling;
mod piece;
mod square;

pub use castling::{CastlingRights, CastlingSide};
pub use piece::{Color, Piece, PieceKind};
pub use square::{File, Rank, Square};
