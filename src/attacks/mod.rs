#[cfg(any(all(feature = "hyperbola", feature = "pext")))]
compile_error!(
    "Features 'hyperbola' and 'pext' are mutually exclusive. Please enable exactly one."
);

#[cfg(feature = "hyperbola")]
mod hyperbola;
mod leapers;
#[cfg(not(feature = "hyperbola"))]
mod magic;

use crate::{Bitboard, Piece, PieceKind, Square};

const fn sliding_attacks(square: Square, occupied: Bitboard, deltas: &[i8]) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;

    let mut i = 0;
    while i < deltas.len() {
        let mut curr = square;
        loop {
            let prev = curr;
            curr = match curr.shift(deltas[i]) {
                Some(square) => square,
                None => break,
            };

            let file_diff = curr.file() as i8 - prev.file() as i8;
            if file_diff.abs() > 2 {
                break;
            }

            attacks.set(curr);

            if occupied.get(curr) {
                break;
            }
        }

        i += 1;
    }

    attacks
}

#[cfg(feature = "hyperbola")]
pub use hyperbola::*;
pub use leapers::*;
#[cfg(not(feature = "hyperbola"))]
pub use magic::*;

/// Returns queen attack squares considering blocking pieces.
///
/// # Arguments
///
/// * `square` - The square the queen is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the queen can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Bitboard, attacks::queen_attacks};
///
/// let attacks = queen_attacks(Square::E4, Bitboard::EMPTY);
/// assert!(!attacks.is_empty());
/// ```
pub const fn queen_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    bishop_attacks(square, occupied).union(rook_attacks(square, occupied))
}

/// Returns attack squares for a piece considering blocking pieces.
///
/// # Arguments
///
/// * `piece` - The piece to get attacks for
/// * `square` - The square the piece is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the piece can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Bitboard, Piece, PieceKind, Color, attacks::piece_attacks};
///
/// let piece = Piece::new(Color::White, PieceKind::Knight);
/// let attacks = piece_attacks(piece, Square::E4, Bitboard::EMPTY);
/// assert_eq!(attacks.popcount(), 8);
/// ```
pub const fn piece_attacks(piece: Piece, square: Square, occupied: Bitboard) -> Bitboard {
    match piece.kind() {
        PieceKind::Pawn => pawn_attacks(square, piece.color()),
        PieceKind::Knight => knight_attacks(square),
        PieceKind::Bishop => bishop_attacks(square, occupied),
        PieceKind::Rook => rook_attacks(square, occupied),
        PieceKind::Queen => queen_attacks(square, occupied),
        PieceKind::King => king_attacks(square),
    }
}
