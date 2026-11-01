use super::Color;
use crate::errors::ParseError;
use core::fmt::{self, Display, Write};
use core::mem::transmute;
use core::str::FromStr;

/// Represents a file on a chess board.
///
/// A file is a column on the chess board, labeled A through H.
/// This enum is niche-optimized as a `repr(u8)` enum, which allows `Option<File>`
/// to occupy the same memory size as `File` itself.
///
/// # Examples
///
/// ```
/// use chess::File;
///
/// let a_file = File::A;
/// let h_file = File::H;
/// ```
#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum File { A, B, C, D, E, F, G, H }

impl File {
    /// The total number of files.
    pub const COUNT: usize = 8;

    /// An array containing all files.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// assert_eq!(File::ALL.len(), 8);
    /// ```
    #[rustfmt::skip]
    pub const ALL: [Self; Self::COUNT] = [Self::A, Self::B, Self::C, Self::D, Self::E, Self::F, Self::G, Self::H];

    /// Converts a u8 to a File.
    ///
    /// # Returns
    ///
    /// Returns `Some(File)` if the value is less than `COUNT` (0-7).
    /// Returns `None` for values 8 or greater.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// assert_eq!(File::from_u8(0), Some(File::A));
    /// assert_eq!(File::from_u8(7), Some(File::H));
    /// assert_eq!(File::from_u8(8), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < Self::COUNT {
            unsafe { Some(Self::from_u8_unchecked(value)) }
        } else {
            None
        }
    }

    /// Converts a u8 to a File without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be less than `COUNT` (0-7). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(File::from_u8_unchecked(0), File::A);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        debug_assert!(value < 8, "invalid value for a File variant");
        unsafe { transmute(value) }
    }

    /// Parses a file from a character.
    ///
    /// Accepts both uppercase and lowercase letters 'a' through 'h'.
    ///
    /// # Returns
    ///
    /// Returns `Some(File)` if the character is valid.
    /// Returns `None` for any other character.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// assert_eq!(File::from_char('a'), Some(File::A));
    /// assert_eq!(File::from_char('A'), Some(File::A));
    /// assert_eq!(File::from_char('h'), Some(File::H));
    /// assert_eq!(File::from_char('i'), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_char(ch: char) -> Option<Self> {
        let value = (ch.to_ascii_lowercase() as u32).wrapping_sub('a' as u32);
        if (value as usize) < Self::COUNT {
            Self::from_u8(value as u8)
        } else {
            None
        }
    }

    /// Converts the file to its character representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// assert_eq!(File::A.to_char(), 'a');
    /// assert_eq!(File::H.to_char(), 'h');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        (b'a' + self as u8) as char
    }

    /// Shifts the file by the given offset.
    ///
    /// # Returns
    ///
    /// Returns `Some(file)` if the shift stays within valid file bounds (A-H).
    /// Returns `None` if the shift would go out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// assert_eq!(File::D.shift(1), Some(File::E));
    /// assert_eq!(File::A.shift(-1), None);
    /// assert_eq!(File::H.shift(1), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn shift(self, offset: i8) -> Option<Self> {
        let value = offset.checked_add(self as i8);
        match value {
            Some(value) => {
                if value >= 0 {
                    Self::from_u8(value as u8)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Shifts the file by the given offset without bounds checking.
    ///
    /// # Safety
    ///
    /// The result must be a valid file (0-7). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::File;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(File::D.shift_unchecked(1), File::E);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const unsafe fn shift_unchecked(self, offset: i8) -> Self {
        unsafe { Self::from_u8_unchecked((self as i8 + offset) as u8) }
    }
}

impl FromStr for File {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Self::from_char(ch).ok_or(ParseError::InvalidFile),
            _ => Err(ParseError::InvalidFile),
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_char())
    }
}

impl_from_enum_for_int!(File => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(File => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents a rank on a chess board.
///
/// A rank is a row on the chess board, labeled 1 through 8.
/// This enum is niche-optimized as a `repr(u8)` enum, which allows `Option<Rank>`
/// to occupy the same memory size as `Rank` itself.
///
/// # Examples
///
/// ```
/// use chess::Rank;
///
/// let rank1 = Rank::R1;
/// let rank8 = Rank::R8;
/// ```
#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank { R1, R2, R3, R4, R5, R6, R7, R8 }

impl Rank {
    /// The total number of ranks.
    pub const COUNT: usize = 8;

    /// An array containing all ranks.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// assert_eq!(Rank::ALL.len(), 8);
    /// ```
    #[rustfmt::skip]
    pub const ALL: [Self; Self::COUNT] = [Self::R1, Self::R2, Self::R3, Self::R4, Self::R5, Self::R6, Self::R7, Self::R8];

    /// Converts a u8 to a Rank.
    ///
    /// # Returns
    ///
    /// Returns `Some(Rank)` if the value is less than `COUNT` (0-7).
    /// Returns `None` for values 8 or greater.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// assert_eq!(Rank::from_u8(0), Some(Rank::R1));
    /// assert_eq!(Rank::from_u8(7), Some(Rank::R8));
    /// assert_eq!(Rank::from_u8(8), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < Self::COUNT {
            unsafe { Some(Self::from_u8_unchecked(value)) }
        } else {
            None
        }
    }

    /// Converts a u8 to a Rank without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be less than `COUNT` (0-7). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(Rank::from_u8_unchecked(0), Rank::R1);
    /// }
    /// ```
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { transmute(value) }
    }

    /// Parses a rank from a character.
    ///
    /// Accepts characters '1' through '8'.
    ///
    /// # Returns
    ///
    /// Returns `Some(Rank)` if the character is valid.
    /// Returns `None` for any other character.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// assert_eq!(Rank::from_char('1'), Some(Rank::R1));
    /// assert_eq!(Rank::from_char('8'), Some(Rank::R8));
    /// assert_eq!(Rank::from_char('9'), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_char(ch: char) -> Option<Self> {
        let value = (ch as u32).wrapping_sub('1' as u32);
        if (value as usize) < Self::COUNT {
            Self::from_u8(value as u8)
        } else {
            None
        }
    }

    /// Converts the rank to its character representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// assert_eq!(Rank::R1.to_char(), '1');
    /// assert_eq!(Rank::R8.to_char(), '8');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        (b'1' + self as u8) as char
    }

    /// Returns the rank relative to the given color.
    ///
    /// For White, this returns the rank as-is. For Black, this mirrors the rank
    /// (R1 becomes R8, R2 becomes R7, etc.). This is useful for color-relative calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Rank, Color};
    ///
    /// assert_eq!(Rank::R2.relative(Color::White), Rank::R2);
    /// assert_eq!(Rank::R2.relative(Color::Black), Rank::R7);
    /// ```
    #[inline]
    #[must_use]
    pub const fn relative(self, color: Color) -> Self {
        unsafe { Self::from_u8_unchecked(self as u8 ^ (0x7 * color as u8)) }
    }

    /// Shifts the rank by the given offset.
    ///
    /// # Returns
    ///
    /// Returns `Some(rank)` if the shift stays within valid rank bounds (R1-R8).
    /// Returns `None` if the shift would go out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// assert_eq!(Rank::R4.shift(1), Some(Rank::R5));
    /// assert_eq!(Rank::R1.shift(-1), None);
    /// assert_eq!(Rank::R8.shift(1), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn shift(self, offset: i8) -> Option<Self> {
        let value = offset.checked_add(self as i8);
        match value {
            Some(value) => {
                if value >= 0 {
                    Self::from_u8(value as u8)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Shifts the rank by the given offset without bounds checking.
    ///
    /// # Safety
    ///
    /// The result must be a valid rank (0-7). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Rank;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(Rank::R4.shift_unchecked(1), Rank::R5);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const unsafe fn shift_unchecked(self, offset: i8) -> Self {
        unsafe { Self::from_u8_unchecked((self as i8 + offset) as u8) }
    }
}

impl FromStr for Rank {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Self::from_char(ch).ok_or(ParseError::InvalidRank),
            _ => Err(ParseError::InvalidRank),
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_char())
    }
}

impl_from_enum_for_int!(Rank => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(Rank => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents a square on a chess board.
///
/// A square is identified by its file (column) and rank (row). There are 64 squares
/// on a standard chess board, labeled A1 through H8. The enum is ordered such that
/// the file is in the lower 3 bits and the rank is in the upper 3 bits.
/// This enum is niche-optimized as a `repr(u8)` enum, which allows `Option<Square>`
/// to occupy the same memory size as `Square` itself.
///
/// # Examples
///
/// ```
/// use chess::Square;
///
/// let a1 = Square::A1;
/// let e4 = Square::E4;
/// let h8 = Square::H8;
/// ```
#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    /// The total number of squares.
    pub const COUNT: usize = 64;

    /// An array containing all squares.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Square;
    ///
    /// assert_eq!(Square::ALL.len(), 64);
    /// ```
    #[rustfmt::skip]
    pub const ALL: [Self; Self::COUNT] = [
        Self::A1, Self::B1, Self::C1, Self::D1, Self::E1, Self::F1, Self::G1, Self::H1,
        Self::A2, Self::B2, Self::C2, Self::D2, Self::E2, Self::F2, Self::G2, Self::H2,
        Self::A3, Self::B3, Self::C3, Self::D3, Self::E3, Self::F3, Self::G3, Self::H3,
        Self::A4, Self::B4, Self::C4, Self::D4, Self::E4, Self::F4, Self::G4, Self::H4,
        Self::A5, Self::B5, Self::C5, Self::D5, Self::E5, Self::F5, Self::G5, Self::H5,
        Self::A6, Self::B6, Self::C6, Self::D6, Self::E6, Self::F6, Self::G6, Self::H6,
        Self::A7, Self::B7, Self::C7, Self::D7, Self::E7, Self::F7, Self::G7, Self::H7,
        Self::A8, Self::B8, Self::C8, Self::D8, Self::E8, Self::F8, Self::G8, Self::H8,
    ];

    /// Creates a new square from a file and rank.
    ///
    /// # Arguments
    ///
    /// * `file` - The file (column) of the square
    /// * `rank` - The rank (row) of the square
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, File, Rank};
    ///
    /// let e4 = Square::new(File::E, Rank::R4);
    /// assert_eq!(e4, Square::E4);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(file: File, rank: Rank) -> Self {
        unsafe { Self::from_u8_unchecked((rank as u8) << 3 | file as u8) }
    }

    /// Converts a u8 to a Square.
    ///
    /// # Returns
    ///
    /// Returns `Some(Square)` if the value is less than `COUNT` (0-63).
    /// Returns `None` for values 64 or greater.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Square;
    ///
    /// assert_eq!(Square::from_u8(0), Some(Square::A1));
    /// assert_eq!(Square::from_u8(63), Some(Square::H8));
    /// assert_eq!(Square::from_u8(64), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if (value as usize) < Self::COUNT {
            unsafe { Some(Self::from_u8_unchecked(value)) }
        } else {
            None
        }
    }

    /// Converts a u8 to a Square without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be less than `COUNT` (0-63). Undefined behavior
    /// may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Square;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(Square::from_u8_unchecked(0), Square::A1);
    /// }
    /// ```
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { transmute(value) }
    }

    /// Returns the file (column) of the square.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, File};
    ///
    /// assert_eq!(Square::E4.file(), File::E);
    /// assert_eq!(Square::A1.file(), File::A);
    /// ```
    #[inline]
    #[must_use]
    pub const fn file(self) -> File {
        unsafe { File::from_u8_unchecked(self as u8 & 0x7) }
    }

    /// Returns the rank (row) of the square.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Rank};
    ///
    /// assert_eq!(Square::E4.rank(), Rank::R4);
    /// assert_eq!(Square::A1.rank(), Rank::R1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn rank(self) -> Rank {
        unsafe { Rank::from_u8_unchecked((self as u8) >> 3) }
    }

    /// Shifts the square by the given offset in the linear index.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the shift stays within valid square bounds (A1-H8).
    /// Returns `None` if the shift would go out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Square;
    ///
    /// assert_eq!(Square::E4.shift(1), Some(Square::F4));
    /// assert_eq!(Square::A1.shift(-1), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn shift(self, shift: i8) -> Option<Self> {
        let value = shift.checked_add(self as i8);
        match value {
            Some(value) => {
                if value >= 0 {
                    Self::from_u8(value as u8)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Shifts the square by the given offset without bounds checking.
    ///
    /// # Safety
    ///
    /// The result must be a valid square (0-63). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Square;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(Square::E4.shift_unchecked(1), Square::F4);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub const unsafe fn shift_unchecked(self, offset: i8) -> Self {
        unsafe { Self::from_u8_unchecked((self as i8 + offset) as u8) }
    }

    /// Returns the square one rank forward for the given color.
    ///
    /// For White, forward means toward higher ranks (R1 -> R2).
    /// For Black, forward means toward lower ranks (R8 -> R7).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Color};
    ///
    /// assert_eq!(Square::E2.forward(Color::White), Square::E3);
    /// assert_eq!(Square::E7.forward(Color::Black), Square::E6);
    /// ```
    #[inline]
    #[must_use]
    pub const fn forward(self, color: Color) -> Self {
        match color {
            Color::White => unsafe { Self::from_u8_unchecked((self as u8 + 8) & 0x3f) },
            Color::Black => unsafe { Self::from_u8_unchecked((self as u8).wrapping_sub(8) & 0x3f) },
        }
    }

    /// Returns the square one rank backward for the given color.
    ///
    /// For White, backward means toward lower ranks (R2 -> R1).
    /// For Black, backward means toward higher ranks (R7 -> R8).
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Color};
    ///
    /// assert_eq!(Square::E3.backward(Color::White), Square::E2);
    /// assert_eq!(Square::E6.backward(Color::Black), Square::E7);
    /// ```
    #[inline]
    #[must_use]
    pub const fn backward(self, color: Color) -> Self {
        self.forward(color.other())
    }

    /// Returns the square relative to the given color.
    ///
    /// For White, this returns the square as-is. For Black, this mirrors
    /// the square vertically (A1 becomes A8, E4 becomes E5, etc.).
    /// This is useful for color-relative calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Color};
    ///
    /// assert_eq!(Square::E4.relative(Color::White), Square::E4);
    /// assert_eq!(Square::E4.relative(Color::Black), Square::E5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn relative(self, color: Color) -> Self {
        unsafe { Self::from_u8_unchecked(self as u8 ^ (0x38 * color as u8)) }
    }
}

impl FromStr for Square {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next(), chars.next()) {
            (Some(file), Some(rank), None) => {
                let file = File::from_char(file).ok_or(ParseError::InvalidSquare)?;
                let rank = Rank::from_char(rank).ok_or(ParseError::InvalidSquare)?;
                Ok(Self::new(file, rank))
            }
            _ => Err(ParseError::InvalidSquare),
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.file().to_char())?;
        f.write_char(self.rank().to_char())
    }
}

impl_from_enum_for_int!(Square => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(Square => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
