/// Bitboard representation for efficient chess board operations.
///
/// This module provides:
/// - `bitboard`: The `Bitboard` struct for representing sets of squares
///
/// # Examples
///
/// ```
/// use chess::{Square, Bitboard};
///
/// let bb = Bitboard::from_square(Square::E4);
/// let rank = Bitboard::from_rank(Square::E4.rank());
/// let intersection = bb.intersect(rank);
/// ```
mod bitboard;
mod geometry;

pub use bitboard::Bitboard;
