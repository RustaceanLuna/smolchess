use super::sliding_attacks;
use crate::{Bitboard, Square};

const fn build_premasks(deltas: (&[i8], &[i8])) -> [(Bitboard, Bitboard); 64] {
    let mut premasks = [(Bitboard::EMPTY, Bitboard::EMPTY); 64];

    let mut curr_square = Some(Square::A1);
    while let Some(square) = curr_square {
        premasks[square as usize].0 = sliding_attacks(square, Bitboard::EMPTY, deltas.0);
        premasks[square as usize].1 = sliding_attacks(square, Bitboard::EMPTY, deltas.1);
        curr_square = square.shift(1);
    }

    premasks
}

const BISHOP_PREMASKS: [(Bitboard, Bitboard); 64] = build_premasks((&[7, -7], &[9, -9]));
const ROOK_PREMASKS: [(Bitboard, Bitboard); 64] = build_premasks((&[1, -1], &[8, -8]));

#[inline]
const fn hyperbola(square: u8, occupied: u64, mask: u64) -> u64 {
    let occupied = occupied & mask;
    let forward = occupied.wrapping_sub(1 << square);
    let backward = occupied.reverse_bits().wrapping_sub(1 << (63 - square));
    (forward ^ backward.reverse_bits()) & mask
}

/// Returns bishop attack squares considering blocking pieces using hyperbola quintessence.
///
/// # Arguments
///
/// * `square` - The square the bishop is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the bishop can move to.
///
/// # Examples
///
/// ```
/// use chess::{Square, Bitboard, attacks::bishop_attacks};
///
/// let attacks = bishop_attacks(Square::E4, Bitboard::EMPTY);
/// assert!(!attacks.is_empty());
/// ```
#[inline]
#[must_use]
pub const fn bishop_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let premasks = BISHOP_PREMASKS[square as usize];
    Bitboard::new(
        hyperbola(square as u8, occupied.value, premasks.0.value)
            | hyperbola(square as u8, occupied.value, premasks.1.value),
    )
}

/// Returns rook attack squares considering blocking pieces using hyperbola quintessence.
///
/// # Arguments
///
/// * `square` - The square the rook is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the rook can move to.
///
/// # Examples
///
/// ```
/// use chess::{Square, Bitboard, attacks::rook_attacks};
///
/// let attacks = rook_attacks(Square::E4, Bitboard::EMPTY);
/// assert!(!attacks.is_empty());
/// ```
#[inline]
#[must_use]
pub const fn rook_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let premasks = ROOK_PREMASKS[square as usize];
    Bitboard::new(
        hyperbola(square as u8, occupied.value, premasks.0.value)
            | hyperbola(square as u8, occupied.value, premasks.1.value),
    )
}
