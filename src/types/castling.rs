use crate::{Color, File};
use core::fmt::{self, Binary, Debug, Display, Write};

/// Represents the side of the board on which castling can occur.
///
/// In chess, castling is a special move involving the king and a rook.
/// Kingside castling (short castling) moves the king two squares toward the h-file,
/// while queenside castling (long castling) moves the king two squares toward the a-file.
///
/// # Examples
///
/// ```
/// use smolchess::CastlingSide;
///
/// let kingside = CastlingSide::Kingside;
/// let queenside = CastlingSide::Queenside;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastlingSide {
    Kingside,
    Queenside,
}

impl CastlingSide {
    /// The total number of castling sides.
    pub const COUNT: usize = 2;

    /// An array containing all castling sides.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::CastlingSide;
    ///
    /// assert_eq!(CastlingSide::ALL.len(), 2);
    /// ```
    pub const ALL: [Self; Self::COUNT] = [Self::Kingside, Self::Queenside];

    /// Returns `Kingside` if `is_kingside` is true, otherwise returns `Queenside`.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::CastlingSide;
    ///
    /// assert_eq!(CastlingSide::kingside(true), CastlingSide::Kingside);
    /// assert_eq!(CastlingSide::kingside(false), CastlingSide::Queenside);
    /// ```
    #[inline]
    #[must_use]
    pub const fn kingside(is_kingside: bool) -> Self {
        if is_kingside {
            Self::Kingside
        } else {
            Self::Queenside
        }
    }

    /// Returns `Queenside` if `is_queenside` is true, otherwise returns `Kingside`.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::CastlingSide;
    ///
    /// assert_eq!(CastlingSide::queenside(true), CastlingSide::Queenside);
    /// assert_eq!(CastlingSide::queenside(false), CastlingSide::Kingside);
    /// ```
    #[inline]
    #[must_use]
    pub const fn queenside(is_queenside: bool) -> Self {
        if is_queenside {
            Self::Queenside
        } else {
            Self::Kingside
        }
    }

    const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Kingside),
            1 => Some(Self::Queenside),
            _ => None,
        }
    }

    /// Returns the opposite castling side.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::CastlingSide;
    ///
    /// assert_eq!(CastlingSide::Kingside.other(), CastlingSide::Queenside);
    /// assert_eq!(CastlingSide::Queenside.other(), CastlingSide::Kingside);
    /// ```
    #[inline]
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Kingside => Self::Queenside,
            Self::Queenside => Self::Kingside,
        }
    }

    /// Returns the file where the king ends up after castling on this side.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingSide, File};
    ///
    /// assert_eq!(CastlingSide::Kingside.king_to_file(), File::G);
    /// assert_eq!(CastlingSide::Queenside.king_to_file(), File::C);
    /// ```
    #[inline]
    #[must_use]
    pub const fn king_to_file(self) -> File {
        match self {
            Self::Kingside => File::G,
            Self::Queenside => File::C,
        }
    }

    /// Returns the file where the rook ends up after castling on this side.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingSide, File};
    ///
    /// assert_eq!(CastlingSide::Kingside.rook_to_file(), File::F);
    /// assert_eq!(CastlingSide::Queenside.rook_to_file(), File::D);
    /// ```
    #[inline]
    #[must_use]
    pub const fn rook_to_file(self) -> File {
        match self {
            Self::Kingside => File::F,
            Self::Queenside => File::D,
        }
    }
}

impl_from_enum_for_int!(CastlingSide => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(CastlingSide => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents the castling rights for both players in a chess position.
///
/// Castling rights track which sides (kingside and queenside) each player
/// can still castle on. This is important because castling rights can be
/// lost when the king or rook moves.
///
/// The internal representation uses a u16 to store both the presence of
/// castling rights and the file positions of the rooks (for chess variants
/// like Chess960 where rooks may not be on their starting files).
///
/// # Examples
///
/// ```
/// use smolchess::{CastlingRights, Color, CastlingSide, File};
///
/// // Create castling rights with standard starting position
/// let rights = CastlingRights::new(
///     Some(File::H),  // White kingside rook on H-file
///     Some(File::A),  // White queenside rook on A-file
///     Some(File::H),  // Black kingside rook on H-file
///     Some(File::A),  // Black queenside rook on A-file
/// );
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastlingRights(u16);

impl CastlingRights {
    /// Castling rights with no castling available for either player.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::CastlingRights;
    ///
    /// assert_eq!(CastlingRights::NONE, CastlingRights::new(None, None, None, None));
    /// ```
    pub const NONE: Self = Self(0);

    /// Creates new castling rights with the specified rook file positions.
    ///
    /// Each parameter is the file where the corresponding rook is located.
    /// `None` indicates that castling on that side is not available.
    ///
    /// # Arguments
    ///
    /// * `white_king` - File of white's kingside rook, or `None` if not available
    /// * `white_queen` - File of white's queenside rook, or `None` if not available
    /// * `black_king` - File of black's kingside rook, or `None` if not available
    /// * `black_queen` - File of black's queenside rook, or `None` if not available
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// // Standard starting position
    /// let rights = CastlingRights::new(
    ///     Some(File::H), Some(File::A),
    ///     Some(File::H), Some(File::A),
    /// );
    ///
    /// // Only white kingside castling available
    /// let rights = CastlingRights::new(
    ///     Some(File::H), None, None, None,
    /// );
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(
        white_king: Option<File>,
        white_queen: Option<File>,
        black_king: Option<File>,
        black_queen: Option<File>,
    ) -> Self {
        let mut this = Self(0);

        if let Some(file) = white_king {
            this.set(Color::White, CastlingSide::Kingside, file);
        }
        if let Some(file) = white_queen {
            this.set(Color::White, CastlingSide::Queenside, file);
        }
        if let Some(file) = black_king {
            this.set(Color::Black, CastlingSide::Kingside, file);
        }
        if let Some(file) = black_queen {
            this.set(Color::Black, CastlingSide::Queenside, file);
        }

        this
    }

    /// Returns the bitflags representation of castling rights.
    ///
    /// The lower 4 bits represent which castling rights are present:
    /// - Bit 0: White kingside
    /// - Bit 1: White queenside
    /// - Bit 2: Black kingside
    /// - Bit 3: Black queenside
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// let rights = CastlingRights::new(
    ///     Some(File::H), Some(File::A), None, None,
    /// );
    /// let flags = rights.bitflags();
    /// ```
    #[inline]
    #[must_use]
    pub const fn bitflags(self) -> u8 {
        self.0 as u8 & 0xf
    }

    /// Gets the file of the rook for the specified color and castling side.
    ///
    /// # Returns
    ///
    /// Returns `Some(file)` if castling is available on that side, where `file`
    /// is the position of the rook. Returns `None` if castling is not available on that side.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// let rights = CastlingRights::new(
    ///     Some(File::H), Some(File::A), None, None,
    /// );
    ///
    /// assert_eq!(rights.get(Color::White, CastlingSide::Kingside), Some(File::H));
    /// assert_eq!(rights.get(Color::White, CastlingSide::Queenside), Some(File::A));
    /// assert_eq!(rights.get(Color::Black, CastlingSide::Kingside), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn get(self, color: Color, side: CastlingSide) -> Option<File> {
        let index = color as u8 + side as u8 * 2;
        let shift = index * 3 + 4;
        let presence = 1 << index;

        if self.0 & presence == 0 {
            return None;
        }

        File::from_u8((self.0 >> shift) as u8 & 0x7)
    }

    /// Sets the castling right for the specified color and side with the given rook file.
    ///
    /// # Arguments
    ///
    /// * `color` - The player color
    /// * `side` - The castling side
    /// * `file` - The file where the rook is located
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// let mut rights = CastlingRights::NONE;
    /// rights.set(Color::White, CastlingSide::Kingside, File::H);
    ///
    /// assert_eq!(rights.get(Color::White, CastlingSide::Kingside), Some(File::H));
    /// ```
    #[inline]
    pub const fn set(&mut self, color: Color, side: CastlingSide, file: File) {
        let index = color as u8 + side as u8 * 2;
        let shift = index * 3 + 4;
        let presence = 1 << index;

        let mask = (0b111 << shift) | presence;
        let value = ((file as u16) << shift) | presence;
        self.0 &= !mask;
        self.0 |= value;
    }

    /// Clears (removes) the castling right for the specified color and side.
    ///
    /// # Arguments
    ///
    /// * `color` - The player color
    /// * `side` - The castling side to clear
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// let mut rights = CastlingRights::new(
    ///     Some(File::H), Some(File::A), None, None,
    /// );
    /// rights.clear(Color::White, CastlingSide::Kingside);
    ///
    /// assert_eq!(rights.get(Color::White, CastlingSide::Kingside), None);
    /// assert_eq!(rights.get(Color::White, CastlingSide::Queenside), Some(File::A));
    /// ```
    #[inline]
    pub const fn clear(&mut self, color: Color, side: CastlingSide) {
        let index = color as u8 + side as u8 * 2;
        let shift = index * 3 + 4;
        let presence = 1 << index;

        let mask = (0b111 << shift) | presence;
        self.0 &= !mask;
    }

    /// Clears castling rights for the specified color if the rook is on the given file.
    ///
    /// This is useful when a rook moves - you can clear the castling right
    /// associated with that rook's file.
    ///
    /// # Arguments
    ///
    /// * `color` - The player color
    /// * `file` - The file to check against rook positions
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingRights, Color, CastlingSide, File};
    ///
    /// let mut rights = CastlingRights::new(
    ///     Some(File::H), Some(File::A), None, None,
    /// );
    /// rights.clear_file(Color::White, File::H);
    ///
    /// assert_eq!(rights.get(Color::White, CastlingSide::Kingside), None);
    /// assert_eq!(rights.get(Color::White, CastlingSide::Queenside), Some(File::A));
    /// ```
    #[inline]
    pub const fn clear_file(&mut self, color: Color, file: File) {
        if let Some(kingside) = self.get(color, CastlingSide::Kingside)
            && file as u8 == kingside as u8
        {
            self.clear(color, CastlingSide::Kingside);
        } else if let Some(queenside) = self.get(color, CastlingSide::Queenside)
            && file as u8 == queenside as u8
        {
            self.clear(color, CastlingSide::Queenside);
        }
    }
}

impl Debug for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CastlingRights({:#})", self)
    }
}

impl Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::NONE {
            return f.write_char('-');
        }

        for color in Color::ALL {
            for side in CastlingSide::ALL {
                if let Some(file) = self.get(color, side) {
                    let mut ch = if f.alternate() {
                        file.to_char()
                    } else {
                        match side {
                            CastlingSide::Kingside => 'k',
                            CastlingSide::Queenside => 'q',
                        }
                    };
                    if color == Color::White {
                        ch = ch.to_ascii_uppercase()
                    }

                    f.write_char(ch)?
                }
            }
        }

        Ok(())
    }
}

impl Binary for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:04b}", self.bitflags())
        } else {
            write!(f, "{:#04b}", self.bitflags())
        }
    }
}

impl From<CastlingRights> for u16 {
    fn from(value: CastlingRights) -> Self {
        value.0
    }
}
