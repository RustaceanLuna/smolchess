use crate::attacks::{bishop_attacks, rook_attacks};
use crate::{Bitboard, Square};

type GeometryTable = [[Bitboard; Square::COUNT]; Square::COUNT];

static RAY_PASS: GeometryTable = {
    let mut table = [[Bitboard::EMPTY; 64]; 64];

    let mut curr_a = Some(Square::A1);
    while let Some(a) = curr_a {
        curr_a = a.shift(1);
        let mut curr_b = Some(Square::A1);
        while let Some(b) = curr_b {
            curr_b = b.shift(1);

            if bishop_attacks(a, Bitboard::EMPTY).get(b) {
                table[a as usize][b as usize] = bishop_attacks(a, Bitboard::EMPTY)
                    .intersect(bishop_attacks(b, a.to_bitboard()).with(b))
                    .with(a);
            }
            if rook_attacks(a, Bitboard::EMPTY).get(b) {
                table[a as usize][b as usize] = rook_attacks(a, Bitboard::EMPTY)
                    .intersect(rook_attacks(b, a.to_bitboard()).with(b))
                    .with(a);
            }
        }
    }

    table
};
static LINE: GeometryTable = {
    let mut table = [[Bitboard::EMPTY; 64]; 64];

    let mut curr_a = Some(Square::A1);
    while let Some(a) = curr_a {
        curr_a = a.shift(1);
        let mut curr_b = Some(Square::A1);
        while let Some(b) = curr_b {
            curr_b = b.shift(1);

            table[a as usize][b as usize] = RAY_PASS[a as usize][b as usize].union(RAY_PASS[b as usize][a as usize]);
        }
    }

    table
};
static BETWEEN: GeometryTable = {
    let mut table = [[Bitboard::EMPTY; 64]; 64];

    let mut curr_a = Some(Square::A1);
    while let Some(a) = curr_a {
        curr_a = a.shift(1);
        let mut curr_b = Some(Square::A1);
        while let Some(b) = curr_b {
            curr_b = b.shift(1);

            table[a as usize][b as usize] = RAY_PASS[a as usize][b as usize]
                .intersect(RAY_PASS[b as usize][a as usize])
                .without(a)
                .without(b);
        }
    }

    table
};

impl Bitboard {
    /// Returns the ray passing through two squares.
    ///
    /// Returns the squares that a piece would pass through when moving
    /// from square `a` to square `b` (excluding the destination).
    ///
    /// # Arguments
    ///
    /// * `a` - The starting square
    /// * `b` - The target square
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares on the ray from `a` through `b`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let ray = Bitboard::ray_pass(Square::E2, Square::E5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn ray_pass(a: Square, b: Square) -> Self {
        RAY_PASS[a as usize][b as usize]
    }

    /// Returns the line passing through two squares.
    ///
    /// Returns all squares on the line connecting two squares,
    /// including both endpoints.
    ///
    /// # Arguments
    ///
    /// * `a` - The first square
    /// * `b` - The second square
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares on the line between `a` and `b`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let line = Bitboard::line(Square::E2, Square::E5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn line(a: Square, b: Square) -> Self {
        LINE[a as usize][b as usize]
    }

    /// Returns the squares between two squares.
    ///
    /// Returns the squares strictly between two squares on the same line,
    /// excluding both endpoints.
    ///
    /// # Arguments
    ///
    /// * `a` - The first square
    /// * `b` - The second square
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares between `a` and `b`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let between = Bitboard::between(Square::E2, Square::E5);
    /// assert!(between.get(Square::E3));
    /// assert!(between.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn between(a: Square, b: Square) -> Self {
        BETWEEN[a as usize][b as usize]
    }

    /// Returns the diagonal passing through a square.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to get the diagonal for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of all squares on the same diagonal.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let diagonal = Bitboard::diagonal(Square::E4);
    /// assert!(diagonal.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn diagonal(square: Square) -> Self {
        let diff = (square.rank() as i8 - square.file() as i8) as u8;
        Self::MAIN_DIAGONAL.shift(if square.rank() as u8 >= square.file() as u8 {
            (diff as i8) * 8
        } else {
            -(diff as i8)
        })
    }

    /// Returns the anti-diagonal passing through a square.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to get the anti-diagonal for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of all squares on the same anti-diagonal.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let anti_diagonal = Bitboard::anti_diagonal(Square::E4);
    /// assert!(anti_diagonal.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn anti_diagonal(square: Square) -> Self {
        let offset = (square.rank() as i8 + square.file() as i8 - 7) << 3;
        Self::MAIN_ANTI_DIAGONAL.shift(offset)
    }
}
