use crate::types::{Color, File, Rank, Square};
use core::fmt::{self, Binary, Debug, Display, LowerHex, UpperHex, Write};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

/// A bitboard representing a set of chess squares.
///
/// A bitboard is a 64-bit integer where each bit corresponds to a square on the
/// chess board. Bit 0 corresponds to A1, bit 1 to B1, ..., bit 63 to H8.
/// Bitboards are used for efficient chess operations like move generation
/// and position evaluation.
///
/// # Examples
///
/// ```
/// use chess::{Square, Bitboard};
///
/// let bb = Bitboard::from_square(Square::E4);
/// assert!(bb.get(Square::E4));
/// ```
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bitboard {
    pub value: u64,
}

impl Bitboard {
    /// An empty bitboard with no squares set.
    pub const EMPTY: Self = Self::new(0);
    /// A bitboard with all 64 squares set.
    pub const ALL: Self = Self::new(0xffffffffffffffff);
    /// The main diagonal (A1-H8).
    pub const MAIN_DIAGONAL: Self = Self::new(0x8040201008040201);
    /// The main anti-diagonal (A8-H1).
    pub const MAIN_ANTI_DIAGONAL: Self = Self::new(0x0102040810204080);
    /// The backranks (rank 1 and rank 8).
    pub const BACKRANKS: Self = Self::new(0xff000000000000ff);
    /// All light-colored squares.
    pub const LIGHT: Self = Self::new(0x55aa55aa55aa55aa);
    /// All dark-colored squares.
    pub const DARK: Self = Self::new(0xaa55aa55aa55aa55);

    /// Creates a new bitboard from a u64 value.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw u64 value
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Bitboard;
    ///
    /// let bb = Bitboard::new(0b1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self { value }
    }

    /// Creates a bitboard with a single square set.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to set
    ///
    /// # Returns
    ///
    /// Returns a bitboard with only the specified square set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_square(square: Square) -> Self {
        Self::new(1 << square as u8)
    }

    /// Creates a bitboard with all squares in a file set.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to set
    ///
    /// # Returns
    ///
    /// Returns a bitboard with all squares in the specified file set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{File, Bitboard};
    ///
    /// let bb = Bitboard::from_file(File::E);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_file(file: File) -> Self {
        Self::new(0x0101010101010101 << file as u8)
    }

    /// Creates a bitboard with all squares in a rank set.
    ///
    /// # Arguments
    ///
    /// * `rank` - The rank to set
    ///
    /// # Returns
    ///
    /// Returns a bitboard with all squares in the specified rank set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Rank, Bitboard};
    ///
    /// let bb = Bitboard::from_rank(Rank::R4);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_rank(rank: Rank) -> Self {
        Self::new(0xff << ((rank as u8) << 3))
    }

    /// Returns true if the bitboard is empty (no squares set).
    ///
    /// # Returns
    ///
    /// Returns true if no squares are set, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Bitboard;
    ///
    /// assert!(Bitboard::EMPTY.is_empty());
    /// assert!(!Bitboard::ALL.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.value == 0
    }

    /// Returns the bitwise complement of the bitboard.
    ///
    /// # Returns
    ///
    /// Returns a bitboard with all bits flipped.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Bitboard;
    ///
    /// let flipped = Bitboard::EMPTY.flip();
    /// assert_eq!(flipped, Bitboard::ALL);
    /// ```
    #[inline]
    #[must_use]
    pub const fn flip(self) -> Self {
        Self::new(!self.value)
    }

    /// Returns the bitwise intersection of two bitboards.
    ///
    /// This is a named method instead of overloading the `&` operator because
    /// Rust does not support const trait implementations for bitwise operators.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard to intersect with
    ///
    /// # Returns
    ///
    /// Returns a bitboard with bits set where both input bitboards have bits set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb1 = Bitboard::from_square(Square::E4);
    /// let bb2 = Bitboard::from_rank(Square::E4.rank());
    /// let intersection = bb1.intersect(bb2);
    /// assert_eq!(intersection, Bitboard::from_square(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn intersect(self, other: Self) -> Self {
        Self::new(self.value & other.value)
    }

    /// Returns the bitwise union of two bitboards.
    ///
    /// This is a named method instead of overloading the `|` operator because
    /// Rust does not support const trait implementations for bitwise operators.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard to union with
    ///
    /// # Returns
    ///
    /// Returns a bitboard with bits set where either input bitboard has bits set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb1 = Bitboard::from_square(Square::E4);
    /// let bb2 = Bitboard::from_square(Square::D4);
    /// let union = bb1.union(bb2);
    /// assert_eq!(union, Bitboard::from_square(Square::E4).union(Bitboard::from_square(Square::D4)));
    /// ```
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self::new(self.value | other.value)
    }

    /// Returns the difference of two bitboards (self & !other).
    ///
    /// This is a named method instead of overloading the `-` operator because
    /// Rust does not support const trait implementations for bitwise operators.
    ///
    /// # Arguments
    ///
    /// * `other` - The bitboard to subtract
    ///
    /// # Returns
    ///
    /// Returns a bitboard with bits set in self but not in other.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb1 = Bitboard::from_rank(Square::E4.rank());
    /// let bb2 = Bitboard::from_square(Square::E4);
    /// let diff = bb1.diff(bb2);
    /// assert!(!diff.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn diff(self, other: Self) -> Self {
        Self::new(self.value & !other.value)
    }

    /// Returns the symmetric difference of two bitboards (self ^ other).
    ///
    /// This is a named method instead of overloading the `^` operator because
    /// Rust does not support const trait implementations for bitwise operators.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard
    ///
    /// # Returns
    ///
    /// Returns a bitboard with bits set where exactly one input bitboard has bits set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb1 = Bitboard::from_square(Square::E4);
    /// let bb2 = Bitboard::from_square(Square::D4);
    /// let sym_diff = bb1.sym_diff(bb2);
    /// assert_eq!(sym_diff.popcount(), 2);
    /// ```
    #[inline]
    #[must_use]
    pub const fn sym_diff(self, other: Self) -> Self {
        Self::new(self.value ^ other.value)
    }

    /// Assigns the intersection of two bitboards to self.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard to intersect with
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb1 = Bitboard::from_rank(Square::E4.rank());
    /// let bb2 = Bitboard::from_square(Square::E4);
    /// bb1.intersect_assign(bb2);
    /// ```
    #[inline]
    pub const fn intersect_assign(&mut self, other: Self) {
        self.value &= other.value
    }

    /// Assigns the union of two bitboards to self.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard to union with
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb1 = Bitboard::from_square(Square::E4);
    /// let bb2 = Bitboard::from_square(Square::D4);
    /// bb1.union_assign(bb2);
    /// ```
    #[inline]
    pub const fn union_assign(&mut self, other: Self) {
        self.value |= other.value
    }

    /// Assigns the difference of two bitboards to self.
    ///
    /// # Arguments
    ///
    /// * `other` - The bitboard to subtract
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb1 = Bitboard::from_rank(Square::E4.rank());
    /// let bb2 = Bitboard::from_square(Square::E4);
    /// bb1.diff_assign(bb2);
    /// ```
    #[inline]
    pub const fn diff_assign(&mut self, other: Self) {
        self.value &= !other.value;
    }

    /// Assigns the symmetric difference of two bitboards to self.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bitboard
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb1 = Bitboard::from_square(Square::E4);
    /// let bb2 = Bitboard::from_square(Square::D4);
    /// bb1.sym_diff_assign(bb2);
    /// ```
    #[inline]
    pub const fn sym_diff_assign(&mut self, other: Self) {
        self.value ^= other.value;
    }

    /// Shifts the bitboard by a specified offset.
    ///
    /// Positive offsets shift up (towards higher ranks), negative shift down.
    /// Wraps around the board edges.
    ///
    /// # Arguments
    ///
    /// * `offset` - The number of squares to shift
    ///
    /// # Returns
    ///
    /// Returns the shifted bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// let shifted = bb.shift(1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn shift(self, offset: i8) -> Self {
        let mask = !((offset >= 0) as i64).wrapping_neg() as u64;
        let positive = self.value.wrapping_shl(offset as u32);
        let negative = self.value.wrapping_shr(-offset as u32);
        Self::new((positive & !mask) | (negative & mask))
    }

    /// Checks if a square is set in the bitboard.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to check
    ///
    /// # Returns
    ///
    /// Returns true if the square is set, false otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// assert!(bb.get(Square::E4));
    /// assert!(!bb.get(Square::D4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn get(self, square: Square) -> bool {
        !self.intersect(square.to_bitboard()).is_empty()
    }

    /// Returns a bitboard with the specified square added.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to add
    ///
    /// # Returns
    ///
    /// Returns a bitboard with the square set.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::EMPTY.with(Square::E4);
    /// assert!(bb.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn with(self, square: Square) -> Self {
        self.union(Self::from_square(square))
    }

    /// Returns a bitboard with the specified square removed.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to remove
    ///
    /// # Returns
    ///
    /// Returns a bitboard with the square cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// let cleared = bb.without(Square::E4);
    /// assert!(cleared.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn without(self, square: Square) -> Self {
        self.intersect(Self::from_square(square).flip())
    }

    /// Returns a bitboard with the specified square toggled.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to toggle
    ///
    /// # Returns
    ///
    /// Returns a bitboard with the square toggled.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::EMPTY.toggled(Square::E4);
    /// assert!(bb.get(Square::E4));
    /// let bb = bb.toggled(Square::E4);
    /// assert!(bb.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn toggled(self, square: Square) -> Self {
        self.sym_diff(Self::from_square(square))
    }

    /// Sets a square in the bitboard.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to set
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb = Bitboard::EMPTY;
    /// bb.set(Square::E4);
    /// assert!(bb.get(Square::E4));
    /// ```
    #[inline]
    pub const fn set(&mut self, square: Square) {
        *self = self.with(square);
    }

    /// Clears a square in the bitboard.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to clear
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb = Bitboard::from_square(Square::E4);
    /// bb.clear(Square::E4);
    /// assert!(bb.is_empty());
    /// ```
    #[inline]
    pub const fn clear(&mut self, square: Square) {
        *self = self.without(square);
    }

    /// Toggles a square in the bitboard.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to toggle
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb = Bitboard::EMPTY;
    /// bb.toggle(Square::E4);
    /// assert!(bb.get(Square::E4));
    /// ```
    #[inline]
    pub const fn toggle(&mut self, square: Square) {
        *self = self.toggled(square)
    }

    /// Returns the least significant bit (LSB) square.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the bitboard is not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// assert_eq!(bb.lsb(), Some(Square::E4));
    /// assert!(Bitboard::EMPTY.lsb().is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn lsb(self) -> Option<Square> {
        Square::from_u8(self.value.trailing_zeros() as u8)
    }

    /// Returns the most significant bit (MSB) square.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the bitboard is not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4);
    /// assert_eq!(bb.msb(), Some(Square::E4));
    /// assert!(Bitboard::EMPTY.msb().is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn msb(self) -> Option<Square> {
        Square::from_u8(63u8.wrapping_sub(self.value.leading_zeros() as u8))
    }

    /// Pops and returns the least significant bit square.
    ///
    /// Removes the LSB from the bitboard and returns it.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the bitboard was not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb = Bitboard::from_square(Square::E4);
    /// assert_eq!(bb.pop_lsb(), Some(Square::E4));
    /// assert!(bb.is_empty());
    /// ```
    #[inline]
    pub const fn pop_lsb(&mut self) -> Option<Square> {
        if let Some(square) = self.lsb() {
            self.value ^= 1 << square as u8;
            Some(square)
        } else {
            None
        }
    }

    /// Pops and returns the most significant bit square.
    ///
    /// Removes the MSB from the bitboard and returns it.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the bitboard was not empty, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let mut bb = Bitboard::from_square(Square::E4);
    /// assert_eq!(bb.pop_msb(), Some(Square::E4));
    /// assert!(bb.is_empty());
    /// ```
    #[inline]
    pub const fn pop_msb(&mut self) -> Option<Square> {
        if let Some(square) = self.msb() {
            self.value ^= 1 << square as u8;
            Some(square)
        } else {
            None
        }
    }

    /// Returns the number of set bits (population count).
    ///
    /// # Returns
    ///
    /// Returns the number of squares set in the bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4).union(Bitboard::from_square(Square::D4));
    /// assert_eq!(bb.popcount(), 2);
    /// ```
    #[inline]
    #[must_use]
    pub const fn popcount(self) -> u8 {
        self.value.count_ones() as u8
    }

    /// Flips the bitboard vertically (ranks 1 and 8 swap).
    ///
    /// # Returns
    ///
    /// Returns the vertically flipped bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E2);
    /// let flipped = bb.flip_vertical();
    /// assert!(flipped.get(Square::E7));
    /// ```
    #[inline]
    #[must_use]
    pub const fn flip_vertical(self) -> Self {
        Self::new(self.value.swap_bytes())
    }

    /// Flips the bitboard horizontally (files A and H swap).
    ///
    /// # Returns
    ///
    /// Returns the horizontally flipped bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::D4);
    /// let flipped = bb.flip_horizontal();
    /// assert!(flipped.get(Square::E4));
    /// ```
    #[inline]
    #[must_use]
    pub const fn flip_horizontal(self) -> Self {
        Self::new(self.value.swap_bytes().reverse_bits())
    }

    /// Returns the bitboard relative to a color.
    ///
    /// For white, returns the bitboard unchanged.
    /// For black, returns the bitboard flipped vertically.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to make relative to
    ///
    /// # Returns
    ///
    /// Returns the color-relative bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard, Color};
    ///
    /// let bb = Bitboard::from_square(Square::E2);
    /// let relative = bb.relative(Color::Black);
    /// assert!(relative.get(Square::E7));
    /// ```
    #[inline]
    #[must_use]
    pub const fn relative(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip_vertical(),
        }
    }

    /// Returns an iterator over all subsets of the bitboard.
    ///
    /// # Returns
    ///
    /// Returns an iterator that yields all subsets of the bitboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Bitboard};
    ///
    /// let bb = Bitboard::from_square(Square::E4).union(Bitboard::from_square(Square::D4));
    /// for subset in bb.subsets() {
    ///     println!("{:?}", subset);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const fn subsets(self) -> Subsets {
        Subsets {
            mask: self.value,
            curr: 0,
            done: false,
        }
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(!self.value)
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.value & rhs.value)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.value &= rhs.value;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.value | rhs.value)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.value |= rhs.value;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::new(self.value ^ rhs.value)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.value ^= rhs.value;
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_lsb()
    }

    fn count(self) -> usize {
        self.popcount() as usize
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.popcount() as usize, Some(self.popcount() as usize))
    }
}

impl DoubleEndedIterator for Bitboard {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pop_msb()
    }
}

impl ExactSizeIterator for Bitboard {
    fn len(&self) -> usize {
        self.popcount() as usize
    }
}

impl Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bitboard({:#x})", self)
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.write_str("  a b c d e f g h\n")?;
        }
        for &rank in Rank::ALL.iter().rev() {
            if f.alternate() {
                f.write_char(rank.to_char())?;
                f.write_char(' ')?;
            }
            for file in File::ALL {
                let ch = if self.get(Square::new(file, rank)) { 'X' } else { '.' };
                f.write_char(ch)?;

                if file != File::H {
                    f.write_char(' ')?;
                }
            }

            if rank != Rank::R1 {
                f.write_char('\n')?;
            }
        }
        Ok(())
    }
}

impl Binary for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:064b}", self.value)
    }
}

impl LowerHex for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.value)
    }
}

impl UpperHex for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016X}", self.value)
    }
}

impl File {
    #[inline]
    #[must_use]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::from_file(self)
    }
}

impl Rank {
    #[inline]
    #[must_use]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::from_rank(self)
    }
}

impl Square {
    #[inline]
    #[must_use]
    pub const fn to_bitboard(self) -> Bitboard {
        Bitboard::from_square(self)
    }
}

/// Iterator over all subsets of a bitboard.
///
/// This struct iterates over all possible subsets of a given bitboard,
/// including the empty subset and the full set.
///
/// # Examples
///
/// ```
/// use chess::{Square, Bitboard};
///
/// let bb = Bitboard::from_square(Square::E4);
/// for subset in bb.subsets() {
///     println!("{:?}", subset);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Subsets {
    mask: u64,
    curr: u64,
    done: bool,
}

impl Iterator for Subsets {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let sub = self.curr;
        if self.curr == self.mask {
            self.done = true;
        } else {
            self.curr = self.curr.wrapping_sub(self.mask) & self.mask;
        }

        Some(Bitboard::new(sub))
    }
}

impl From<File> for Bitboard {
    fn from(value: File) -> Self {
        value.to_bitboard()
    }
}

impl From<Rank> for Bitboard {
    fn from(value: Rank) -> Self {
        value.to_bitboard()
    }
}

impl From<Square> for Bitboard {
    fn from(value: Square) -> Self {
        value.to_bitboard()
    }
}

impl From<u64> for Bitboard {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<Bitboard> for u64 {
    fn from(value: Bitboard) -> Self {
        value.value
    }
}
