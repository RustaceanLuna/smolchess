use crate::errors::ParseError;
use core::fmt::{self, Display, Write};
use core::mem::transmute;
use core::ops::Not;
use core::str::FromStr;

/// Represents a color in chess.
///
/// This enum represents the two colors in chess: White and Black.
/// This enum is niche-optimized as a `repr(u8)` enum, which allows `Option<Color>`
/// to occupy the same memory size as `Color` itself.
///
/// # Examples
///
/// ```
/// use chess::Color;
///
/// let white = Color::White;
/// let black = Color::Black;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// The total number of colors.
    pub const COUNT: usize = 2;

    /// An array containing all colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::ALL.len(), 2);
    /// ```
    pub const ALL: [Self; Self::COUNT] = [Self::White, Self::Black];

    /// Returns `White` if `is_white` is true, otherwise returns `Black`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::from_white(true), Color::White);
    /// assert_eq!(Color::from_white(false), Color::Black);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_white(is_white: bool) -> Self {
        if is_white { Self::White } else { Self::Black }
    }

    /// Returns `Black` if `is_black` is true, otherwise returns `White`.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::from_black(true), Color::Black);
    /// assert_eq!(Color::from_black(false), Color::White);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_black(is_black: bool) -> Self {
        if is_black { Self::Black } else { Self::White }
    }

    const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::White),
            1 => Some(Self::Black),
            _ => None,
        }
    }

    /// Parses a color from a character.
    ///
    /// # Returns
    ///
    /// Returns `Some(Color)` if the character is 'w' or 'b'.
    /// Returns `None` for any other character.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::from_char('w'), Some(Color::White));
    /// assert_eq!(Color::from_char('b'), Some(Color::Black));
    /// assert_eq!(Color::from_char('x'), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_char(ch: char) -> Option<Self> {
        match ch {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    /// Returns the opposite color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::White.other(), Color::Black);
    /// assert_eq!(Color::Black.other(), Color::White);
    /// ```
    #[inline]
    #[must_use]
    pub const fn other(self) -> Self {
        Self::from_black(matches!(self, Self::White))
    }

    /// Converts the color to its character representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// assert_eq!(Color::White.to_char(), 'w');
    /// assert_eq!(Color::Black.to_char(), 'b');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    /// Creates a pawn of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.pawn(), Piece::WhitePawn);
    /// assert_eq!(Color::Black.pawn(), Piece::BlackPawn);
    /// ```
    #[inline]
    #[must_use]
    pub const fn pawn(self) -> Piece {
        Piece::new(self, PieceKind::Pawn)
    }

    /// Creates a knight of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.knight(), Piece::WhiteKnight);
    /// assert_eq!(Color::Black.knight(), Piece::BlackKnight);
    /// ```
    #[inline]
    #[must_use]
    pub const fn knight(self) -> Piece {
        Piece::new(self, PieceKind::Knight)
    }

    /// Creates a bishop of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.bishop(), Piece::WhiteBishop);
    /// assert_eq!(Color::Black.bishop(), Piece::BlackBishop);
    /// ```
    #[inline]
    #[must_use]
    pub const fn bishop(self) -> Piece {
        Piece::new(self, PieceKind::Bishop)
    }

    /// Creates a rook of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.rook(), Piece::WhiteRook);
    /// assert_eq!(Color::Black.rook(), Piece::BlackRook);
    /// ```
    #[inline]
    #[must_use]
    pub const fn rook(self) -> Piece {
        Piece::new(self, PieceKind::Rook)
    }

    /// Creates a queen of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.queen(), Piece::WhiteQueen);
    /// assert_eq!(Color::Black.queen(), Piece::BlackQueen);
    /// ```
    #[inline]
    #[must_use]
    pub const fn queen(self) -> Piece {
        Piece::new(self, PieceKind::Queen)
    }

    /// Creates a king of this color.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, Piece};
    ///
    /// assert_eq!(Color::White.king(), Piece::WhiteKing);
    /// assert_eq!(Color::Black.king(), Piece::BlackKing);
    /// ```
    #[inline]
    #[must_use]
    pub const fn king(self) -> Piece {
        Piece::new(self, PieceKind::King)
    }

    /// Flips the color to its opposite.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Color;
    ///
    /// let mut color = Color::White;
    /// color.flip();
    /// assert_eq!(color, Color::Black);
    /// ```
    pub const fn flip(&mut self) {
        *self = self.other();
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        self.other()
    }
}

impl FromStr for Color {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Self::from_char(ch).ok_or(ParseError::InvalidColor),
            _ => Err(ParseError::InvalidColor),
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_char())
    }
}

impl_from_enum_for_int!(Color => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(Color => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents the kind of a chess piece.
///
/// This enum represents the six kinds of chess pieces: King, Queen, Rook,
/// Bishop, Knight, and Pawn. This enum is niche-optimized as a `repr(u8)` enum,
/// which allows `Option<PieceKind>` to occupy the same memory size as `PieceKind` itself.
///
/// # Examples
///
/// ```
/// use chess::PieceKind;
///
/// let king = PieceKind::King;
/// let pawn = PieceKind::Pawn;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    /// The total number of piece kinds.
    pub const COUNT: usize = 6;

    /// An array containing all piece kinds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// assert_eq!(PieceKind::ALL.len(), 6);
    /// ```
    pub const ALL: [Self; Self::COUNT] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    /// Converts a u8 to a PieceKind.
    ///
    /// # Returns
    ///
    /// Returns `Some(PieceKind)` if the value is less than `COUNT` (0-5).
    /// Returns `None` for values 6 or greater.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// assert_eq!(PieceKind::from_u8(0), Some(PieceKind::Pawn));
    /// assert_eq!(PieceKind::from_u8(5), Some(PieceKind::King));
    /// assert_eq!(PieceKind::from_u8(6), None);
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

    /// Converts a u8 to a PieceKind without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be less than `COUNT` (0-5). Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(PieceKind::from_u8_unchecked(0), PieceKind::Pawn);
    /// }
    /// ```
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { transmute(value) }
    }

    /// Parses a piece kind from a character.
    ///
    /// Accepts both uppercase and lowercase letters:
    /// - 'P' or 'p' for Pawn
    /// - 'N' or 'n' for Knight
    /// - 'B' or 'b' for Bishop
    /// - 'R' or 'r' for Rook
    /// - 'Q' or 'q' for Queen
    /// - 'K' or 'k' for King
    ///
    /// # Returns
    ///
    /// Returns `Some(PieceKind)` if the character is valid.
    /// Returns `None` for any other character.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// assert_eq!(PieceKind::from_char('P'), Some(PieceKind::Pawn));
    /// assert_eq!(PieceKind::from_char('p'), Some(PieceKind::Pawn));
    /// assert_eq!(PieceKind::from_char('N'), Some(PieceKind::Knight));
    /// assert_eq!(PieceKind::from_char('x'), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_char(value: char) -> Option<Self> {
        match value {
            'P' | 'p' => Some(Self::Pawn),
            'N' | 'n' => Some(Self::Knight),
            'B' | 'b' => Some(Self::Bishop),
            'R' | 'r' => Some(Self::Rook),
            'Q' | 'q' => Some(Self::Queen),
            'K' | 'k' => Some(Self::King),
            _ => None,
        }
    }

    /// Converts the piece kind to its lowercase character representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// assert_eq!(PieceKind::Pawn.to_lower_char(), 'p');
    /// assert_eq!(PieceKind::Knight.to_lower_char(), 'n');
    /// assert_eq!(PieceKind::King.to_lower_char(), 'k');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_lower_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }

    /// Converts the piece kind to its uppercase character representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::PieceKind;
    ///
    /// assert_eq!(PieceKind::Pawn.to_upper_char(), 'P');
    /// assert_eq!(PieceKind::Knight.to_upper_char(), 'N');
    /// assert_eq!(PieceKind::King.to_upper_char(), 'K');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_upper_char(self) -> char {
        match self {
            Self::Pawn => 'P',
            Self::Knight => 'N',
            Self::Bishop => 'B',
            Self::Rook => 'R',
            Self::Queen => 'Q',
            Self::King => 'K',
        }
    }

    /// Creates a white piece of this kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{PieceKind, Piece};
    ///
    /// assert_eq!(PieceKind::Pawn.white(), Piece::WhitePawn);
    /// assert_eq!(PieceKind::King.white(), Piece::WhiteKing);
    /// ```
    #[inline]
    #[must_use]
    pub const fn white(self) -> Piece {
        Piece::new(Color::White, self)
    }

    /// Creates a black piece of this kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{PieceKind, Piece};
    ///
    /// assert_eq!(PieceKind::Pawn.black(), Piece::BlackPawn);
    /// assert_eq!(PieceKind::King.black(), Piece::BlackKing);
    /// ```
    #[inline]
    #[must_use]
    pub const fn black(self) -> Piece {
        Piece::new(Color::Black, self)
    }
}

impl FromStr for PieceKind {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Self::from_char(ch).ok_or(ParseError::InvalidPieceKind),
            _ => Err(ParseError::InvalidPieceKind),
        }
    }
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_upper_char())
    }
}

impl_from_enum_for_int!(PieceKind => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(PieceKind => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents a complete chess piece with both color and kind.
///
/// This enum combines a `Color` and a `PieceKind` to represent all 12 possible
/// chess pieces (6 kinds × 2 colors). The variants are ordered such that the
/// color is in the least significant bit and the piece kind is in the remaining bits.
///
/// # Implementation Note
///
/// This enum is niche-optimized as a `repr(u8)` enum, which allows `Option<Piece>`
/// to occupy the same memory size as `Piece` itself.
///
/// # Examples
///
/// ```
/// use chess::Piece;
///
/// let white_pawn = Piece::WhitePawn;
/// let black_king = Piece::BlackKing;
/// ```
#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    WhitePawn,   BlackPawn,
    WhiteKnight, BlackKnight,
    WhiteBishop, BlackBishop,
    WhiteRook,   BlackRook,
    WhiteQueen,  BlackQueen,
    WhiteKing,   BlackKing,
}

impl Piece {
    /// The total number of pieces.
    pub const COUNT: usize = 12;

    /// An array containing all pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Piece;
    ///
    /// assert_eq!(Piece::ALL.len(), 12);
    /// ```
    #[rustfmt::skip]
    pub const ALL: [Self; Self::COUNT] = [
        Self::WhitePawn,   Self::BlackPawn,
        Self::WhiteKnight, Self::BlackKnight,
        Self::WhiteBishop, Self::BlackBishop,
        Self::WhiteRook,   Self::BlackRook,
        Self::WhiteQueen,  Self::BlackQueen,
        Self::WhiteKing,   Self::BlackKing,
    ];

    /// Creates a new piece from a color and kind.
    ///
    /// # Arguments
    ///
    /// * `color` - The color of the piece
    /// * `kind` - The kind of piece
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Piece, Color, PieceKind};
    ///
    /// let white_pawn = Piece::new(Color::White, PieceKind::Pawn);
    /// let black_king = Piece::new(Color::Black, PieceKind::King);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        unsafe { Self::from_u8_unchecked((kind as u8) << 1 | color as u8) }
    }

    /// Converts a u8 to a Piece.
    ///
    /// # Returns
    ///
    /// Returns `Some(Piece)` if the value is less than or equal to `COUNT` (0-11).
    /// Returns `None` for values 12 or greater.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Piece;
    ///
    /// assert_eq!(Piece::from_u8(0), Some(Piece::WhitePawn));
    /// assert_eq!(Piece::from_u8(11), Some(Piece::BlackKing));
    /// assert_eq!(Piece::from_u8(12), None);
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

    /// Converts a u8 to a Piece without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be less than or equal to `COUNT` (0-11). Undefined behavior
    /// may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Piece;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(Piece::from_u8_unchecked(0), Piece::WhitePawn);
    /// }
    /// ```
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { transmute(value) }
    }

    /// Parses a piece from a character.
    ///
    /// Uppercase letters represent white pieces, lowercase letters represent black pieces:
    /// - 'P'/'p' for Pawn, 'N'/'n' for Knight, 'B'/'b' for Bishop
    /// - 'R'/'r' for Rook, 'Q'/'q' for Queen, 'K'/'k' for King
    ///
    /// # Returns
    ///
    /// Returns `Some(Piece)` if the character is valid.
    /// Returns `None` for any other character.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Piece;
    ///
    /// assert_eq!(Piece::from_char('P'), Some(Piece::WhitePawn));
    /// assert_eq!(Piece::from_char('p'), Some(Piece::BlackPawn));
    /// assert_eq!(Piece::from_char('K'), Some(Piece::WhiteKing));
    /// assert_eq!(Piece::from_char('x'), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_char(value: char) -> Option<Self> {
        match value {
            'P' => Some(Self::WhitePawn),
            'p' => Some(Self::BlackPawn),
            'N' => Some(Self::WhiteKnight),
            'n' => Some(Self::BlackKnight),
            'B' => Some(Self::WhiteBishop),
            'b' => Some(Self::BlackBishop),
            'R' => Some(Self::WhiteRook),
            'r' => Some(Self::BlackRook),
            'Q' => Some(Self::WhiteQueen),
            'q' => Some(Self::BlackQueen),
            'K' => Some(Self::WhiteKing),
            'k' => Some(Self::BlackKing),
            _ => None,
        }
    }

    /// Converts the piece to its character representation.
    ///
    /// Uppercase for white pieces, lowercase for black pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::Piece;
    ///
    /// assert_eq!(Piece::WhitePawn.to_char(), 'P');
    /// assert_eq!(Piece::BlackPawn.to_char(), 'p');
    /// assert_eq!(Piece::WhiteKing.to_char(), 'K');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_char(self) -> char {
        match self {
            Self::WhitePawn => 'P',
            Self::BlackPawn => 'p',
            Self::WhiteKnight => 'N',
            Self::BlackKnight => 'n',
            Self::WhiteBishop => 'B',
            Self::BlackBishop => 'b',
            Self::WhiteRook => 'R',
            Self::BlackRook => 'r',
            Self::WhiteQueen => 'Q',
            Self::BlackQueen => 'q',
            Self::WhiteKing => 'K',
            Self::BlackKing => 'k',
        }
    }

    /// Returns the color of the piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Piece, Color};
    ///
    /// assert_eq!(Piece::WhitePawn.color(), Color::White);
    /// assert_eq!(Piece::BlackKing.color(), Color::Black);
    /// ```
    #[inline]
    #[must_use]
    pub const fn color(self) -> Color {
        Color::from_white(self as u8 & 1 == 0)
    }

    /// Returns the kind of the piece.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Piece, PieceKind};
    ///
    /// assert_eq!(Piece::WhitePawn.kind(), PieceKind::Pawn);
    /// assert_eq!(Piece::BlackKing.kind(), PieceKind::King);
    /// ```
    #[inline]
    #[must_use]
    pub const fn kind(self) -> PieceKind {
        unsafe { PieceKind::from_u8_unchecked(self as u8 >> 1) }
    }
}

impl FromStr for Piece {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Self::from_char(ch).ok_or(ParseError::InvalidPiece),
            _ => Err(ParseError::InvalidPiece),
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(self.to_char())
    }
}

impl_from_enum_for_int!(Piece => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(Piece => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
