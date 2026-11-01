use super::sliding_attacks;
use crate::{Bitboard, Color, File, Square};

type AttacksTable = [Bitboard; Square::COUNT];

const fn build_leaping_attacks(deltas: &[i8]) -> AttacksTable {
    let mut table = [Bitboard::EMPTY; Square::COUNT];

    let mut curr_square = Some(Square::A1);
    while let Some(square) = curr_square {
        table[square as usize] = sliding_attacks(square, Bitboard::ALL, deltas);
        curr_square = square.shift(1);
    }

    table
}

const WHITE_PAWN_ATTACKS: AttacksTable = build_leaping_attacks(&[7, 9]);
const BLACK_PAWN_ATTACKS: AttacksTable = build_leaping_attacks(&[-7, -9]);
const KNIGHT_ATTACKS: AttacksTable = build_leaping_attacks(&[6, 10, 15, 17, -6, -10, -15, -17]);
const KING_ATTACKS: AttacksTable = build_leaping_attacks(&[1, 7, 8, 9, -1, -7, -8, -9]);

/// Returns pawn attack squares for a given square and color.
///
/// # Arguments
///
/// * `square` - The square the pawn is on
/// * `color` - The color of the pawn
///
/// # Returns
///
/// Returns a bitboard of squares the pawn can attack.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Color, attacks::pawn_attacks};
///
/// let attacks = pawn_attacks(Square::E4, Color::White);
/// assert!(attacks.get(Square::D5) || attacks.get(Square::F5));
/// ```
#[inline]
#[must_use]
pub const fn pawn_attacks(square: Square, color: Color) -> Bitboard {
    match color {
        Color::White => WHITE_PAWN_ATTACKS[square as usize],
        Color::Black => BLACK_PAWN_ATTACKS[square as usize],
    }
}

/// Returns pawn attack squares for a bitboard of pawns.
///
/// # Arguments
///
/// * `bitboard` - A bitboard of pawn positions
/// * `color` - The color of the pawns
///
/// # Returns
///
/// Returns a bitboard of squares the pawns can attack.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Color, Bitboard, attacks::pawn_attacks_bb};
///
/// let pawns = Bitboard::from_square(Square::E4);
/// let attacks = pawn_attacks_bb(pawns, Color::White);
/// assert!(!attacks.is_empty());
/// ```
#[inline]
#[must_use]
pub const fn pawn_attacks_bb(bitboard: Bitboard, color: Color) -> Bitboard {
    const FILE_A: Bitboard = File::A.to_bitboard();
    const FILE_H: Bitboard = File::H.to_bitboard();

    let (left, right) = match color {
        Color::White => (7, 9),
        Color::Black => (-9, -7),
    };
    bitboard
        .diff(FILE_A)
        .shift(left)
        .union(bitboard.diff(FILE_H).shift(right))
}

/// Returns knight attack squares for a given square.
///
/// # Arguments
///
/// * `square` - The square the knight is on
///
/// # Returns
///
/// Returns a bitboard of squares the knight can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, attacks::knight_attacks};
///
/// let attacks = knight_attacks(Square::E4);
/// assert_eq!(attacks.popcount(), 8);
/// ```
#[inline]
#[must_use]
pub const fn knight_attacks(square: Square) -> Bitboard {
    KNIGHT_ATTACKS[square as usize]
}

/// Returns king attack squares for a given square.
///
/// # Arguments
///
/// * `square` - The square the king is on
///
/// # Returns
///
/// Returns a bitboard of squares the king can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, attacks::king_attacks};
///
/// let attacks = king_attacks(Square::E4);
/// assert_eq!(attacks.popcount(), 8);
/// ```
#[inline]
#[must_use]
pub const fn king_attacks(square: Square) -> Bitboard {
    KING_ATTACKS[square as usize]
}
