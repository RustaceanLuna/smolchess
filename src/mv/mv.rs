use core::fmt::Debug;
use core::mem::transmute;
use core::num::NonZeroU16;

use crate::attacks::{pawn_attacks, piece_attacks};
use crate::errors::MoveError;
use crate::{Bitboard, CastlingSide, CoreBoard, PieceKind, Rank, Square};

/// Represents the kind of chess move.
///
/// This enum categorizes moves into different types:
/// - **Quiet moves**: Non-capturing moves (Quiet, DoublePush, KingCastle, QueenCastle)
/// - **Captures**: Moves that capture an opponent's piece (Capture, EnPassant)
/// - **Promotions**: Pawn promotion moves (KnightPromotion, BishopPromotion, RookPromotion, QueenPromotion)
/// - **Promotion Captures**: Pawn promotion moves that also capture (KnightPromoCapture, etc.)
///
/// Note that castling moves are represented as the king capturing the rook.
///
/// # Examples
///
/// ```
/// use smolchess::mv::MoveKind;
///
/// let quiet = MoveKind::Quiet;
/// let capture = MoveKind::Capture;
/// let promotion = MoveKind::QueenPromotion;
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveKind {
    // Quiets
    Quiet = 0b0000,
    DoublePush = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,

    // Captures
    Capture = 0b0100,
    EnPassant = 0b0101,

    // Promotions
    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,

    // Promotion Captures
    KnightPromoCapture = 0b1100,
    BishopPromoCapture = 0b1101,
    RookPromoCapture = 0b1110,
    QueenPromoCapture = 0b1111,
}

impl MoveKind {
    /// The total number of move kinds.
    pub const COUNT: usize = 14;

    /// An array containing all move kinds.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert_eq!(MoveKind::ALL.len(), 14);
    /// ```
    pub const ALL: [Self; 14] = [
        Self::Quiet,
        Self::DoublePush,
        Self::KingCastle,
        Self::QueenCastle,
        Self::Capture,
        Self::EnPassant,
        Self::KnightPromotion,
        Self::BishopPromotion,
        Self::RookPromotion,
        Self::QueenPromotion,
        Self::KnightPromoCapture,
        Self::BishopPromoCapture,
        Self::RookPromoCapture,
        Self::QueenPromoCapture,
    ];

    /// Converts a u8 to a MoveKind.
    ///
    /// # Returns
    ///
    /// Returns `Some(MoveKind)` if the value is a valid move kind (0-15, excluding 0b0110 and 0b0111).
    /// Returns `None` for invalid values.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert_eq!(MoveKind::from_u8(0b0000), Some(MoveKind::Quiet));
    /// assert_eq!(MoveKind::from_u8(0b0110), None);
    /// assert_eq!(MoveKind::from_u8(16), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if value < 16 && value != 0b0110 && value != 0b0111 {
            unsafe { Some(Self::from_u8_unchecked(value)) }
        } else {
            None
        }
    }

    /// Converts a u8 to a MoveKind without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be a valid move kind (0-15, excluding 0b0110 and 0b0111).
    /// Undefined behavior may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// // Safe usage
    /// unsafe {
    ///     assert_eq!(MoveKind::from_u8_unchecked(0b0000), MoveKind::Quiet);
    /// }
    /// ```
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        debug_assert!(
            value < 16 && value != 0b0110 && value != 0b0111,
            "invalid value for a MoveKind variant"
        );
        unsafe { transmute(value) }
    }

    /// Creates a castling move kind from the castling side.
    ///
    /// # Arguments
    ///
    /// * `side` - The castling side (Kingside or Queenside)
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingSide, mv::MoveKind};
    ///
    /// assert_eq!(MoveKind::from_castling(CastlingSide::Kingside), MoveKind::KingCastle);
    /// assert_eq!(MoveKind::from_castling(CastlingSide::Queenside), MoveKind::QueenCastle);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_castling(side: CastlingSide) -> Self {
        unsafe { Self::from_u8_unchecked(0b0010 | side as u8) }
    }

    /// Creates a promotion move kind from the piece kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The piece kind to promote to (Knight, Bishop, Rook, or Queen)
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{PieceKind, mv::MoveKind};
    ///
    /// assert_eq!(MoveKind::from_promotion(PieceKind::Queen), MoveKind::QueenPromotion);
    /// assert_eq!(MoveKind::from_promotion(PieceKind::Knight), MoveKind::KnightPromotion);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_promotion(kind: PieceKind) -> Self {
        unsafe { Self::from_u8_unchecked(0b1000 | (kind as u8 - PieceKind::Knight as u8)) }
    }

    /// Creates a promotion capture move kind from the piece kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The piece kind to promote to (Knight, Bishop, Rook, or Queen)
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{PieceKind, mv::MoveKind};
    ///
    /// assert_eq!(MoveKind::from_promo_capture(PieceKind::Queen), MoveKind::QueenPromoCapture);
    /// assert_eq!(MoveKind::from_promo_capture(PieceKind::Knight), MoveKind::KnightPromoCapture);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from_promo_capture(kind: PieceKind) -> Self {
        unsafe { Self::from_u8_unchecked(0b1100 | (kind as u8 - PieceKind::Knight as u8)) }
    }

    /// Returns true if this is a castling move.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert!(MoveKind::KingCastle.is_castling());
    /// assert!(MoveKind::QueenCastle.is_castling());
    /// assert!(!MoveKind::Quiet.is_castling());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_castling(self) -> bool {
        self as u8 & 0b1110 == 0b0010
    }

    /// Returns true if this is a capture move.
    ///
    /// This includes regular captures, en passant, and promotion captures.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert!(MoveKind::Capture.is_capture());
    /// assert!(MoveKind::EnPassant.is_capture());
    /// assert!(MoveKind::QueenPromoCapture.is_capture());
    /// assert!(!MoveKind::Quiet.is_capture());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        self as u8 & 0b0100 != 0
    }

    /// Returns true if this is a promotion move.
    ///
    /// This includes both regular promotions and promotion captures.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert!(MoveKind::QueenPromotion.is_promotion());
    /// assert!(MoveKind::QueenPromoCapture.is_promotion());
    /// assert!(!MoveKind::Quiet.is_promotion());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        self as u8 & 0b1000 != 0
    }

    /// Returns true if this is a tactical move.
    ///
    /// Tactical moves are captures or promotions.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::MoveKind;
    ///
    /// assert!(MoveKind::Capture.is_tactical());
    /// assert!(MoveKind::QueenPromotion.is_tactical());
    /// assert!(!MoveKind::Quiet.is_tactical());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_tactical(self) -> bool {
        self.is_capture() || self.is_promotion()
    }

    /// Returns the castling side if this is a castling move.
    ///
    /// # Returns
    ///
    /// Returns `Some(CastlingSide)` if this is a castling move.
    /// Returns `None` for non-castling moves.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{CastlingSide, mv::MoveKind};
    ///
    /// assert_eq!(MoveKind::KingCastle.castling_side(), Some(CastlingSide::Kingside));
    /// assert_eq!(MoveKind::Quiet.castling_side(), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn castling_side(self) -> Option<CastlingSide> {
        if self.is_castling() {
            Some(CastlingSide::kingside(matches!(self, Self::KingCastle)))
        } else {
            None
        }
    }

    /// Returns the promotion piece kind if this is a promotion move.
    ///
    /// # Returns
    ///
    /// Returns `Some(PieceKind)` if this is a promotion move, indicating the piece
    /// the pawn promotes to. Returns `None` for non-promotion moves.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{PieceKind, mv::MoveKind};
    ///
    /// assert_eq!(MoveKind::QueenPromotion.promotion(), Some(PieceKind::Queen));
    /// assert_eq!(MoveKind::KnightPromoCapture.promotion(), Some(PieceKind::Knight));
    /// assert_eq!(MoveKind::Quiet.promotion(), None);
    /// ```
    #[inline]
    #[must_use]
    pub const fn promotion(self) -> Option<PieceKind> {
        if self.is_promotion() {
            unsafe {
                Some(PieceKind::from_u8_unchecked(
                    (self as u8 & 0x3) + PieceKind::Knight as u8,
                ))
            }
        } else {
            None
        }
    }
}

impl_from_enum_for_int!(MoveKind => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);
impl_try_from_int_for_enum!(MoveKind => i8, u8, i16, u16, i32, u32, i64, u64, i128, u128);

/// Represents a chess move.
///
/// A move is represented as a compact 16-bit value containing:
/// - Bits 0-5: The source square (from)
/// - Bits 6-11: The destination square (to)
/// - Bits 12-15: The move kind
///
/// For castling moves, the `to` square represents the rook's position (king captures rook).
///
/// This struct is niche-optimized using `NonZeroU16`, which allows `Option<Move>` to
/// occupy the same memory size as `Move` itself.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, mv::{Move, MoveKind}};
///
/// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move(NonZeroU16);

impl Move {
    /// Creates a new move from source, destination, and kind.
    ///
    /// # Arguments
    ///
    /// * `from` - The source square
    /// * `to` - The destination square
    /// * `kind` - The kind of move
    ///
    /// # Returns
    ///
    /// Returns `Some(Move)` if `from` and `to` are different squares.
    /// Returns `None` if `from` and `to` are the same square.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush);
    /// assert!(mv.is_some());
    ///
    /// let invalid = Move::new(Square::E2, Square::E2, MoveKind::Quiet);
    /// assert!(invalid.is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(from: Square, to: Square, kind: MoveKind) -> Option<Self> {
        if from as u8 != to as u8 {
            unsafe { Some(Self::new_unchecked(from, to, kind)) }
        } else {
            None
        }
    }

    /// Creates a new move without checking if from and to are different.
    ///
    /// # Safety
    ///
    /// `from` and `to` must be different squares. Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `from` - The source square
    /// * `to` - The destination square
    /// * `kind` - The kind of move
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// // Safe usage
    /// unsafe {
    ///     let mv = Move::new_unchecked(Square::E2, Square::E4, MoveKind::DoublePush);
    /// }
    /// ```
    pub const unsafe fn new_unchecked(from: Square, to: Square, kind: MoveKind) -> Self {
        debug_assert!(
            from as u8 != to as u8,
            "from and to cannot be the same Square for a Move"
        );
        unsafe { Self::from_u16((from as u16) | ((to as u16) << 6) | ((kind as u16) << 12)) }
    }

    /// Creates a move from a u16 value without bounds checking.
    ///
    /// # Safety
    ///
    /// The value must be a valid non-zero u16 representing a move.
    /// Undefined behavior may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::mv::Move;
    ///
    /// // Safe usage
    /// unsafe {
    ///     let mv = Move::from_u16(1);
    /// }
    /// ```
    pub const unsafe fn from_u16(value: u16) -> Self {
        unsafe { Self(NonZeroU16::new_unchecked(value)) }
    }

    /// Converts the move to its u16 representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// let value = mv.to_u16();
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self.0.get()
    }

    /// Returns the source square of the move.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// assert_eq!(mv.from(), Square::E2);
    /// ```
    #[inline]
    #[must_use]
    pub const fn from(self) -> Square {
        unsafe { Square::from_u8_unchecked((self.to_u16() & 0x3F) as u8) }
    }

    /// Returns the destination square of the move.
    ///
    /// For castling moves, this is the rook's square (king captures rook).
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// assert_eq!(mv.to(), Square::E4);
    /// ```
    #[inline]
    #[must_use]
    pub const fn to(self) -> Square {
        unsafe { Square::from_u8_unchecked(((self.to_u16() >> 6) & 0x3F) as u8) }
    }

    /// Returns the kind of move.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// assert_eq!(mv.kind(), MoveKind::DoublePush);
    /// ```
    #[inline]
    #[must_use]
    pub const fn kind(self) -> MoveKind {
        unsafe { MoveKind::from_u8_unchecked((self.to_u16() >> 12) as u8) }
    }
}

impl CoreBoard {
    const fn check_pseudolegal_castling(&self, mv: Move) -> Result<(), MoveError> {
        let us = self.side_to_move();
        let side = mv.kind().castling_side().unwrap();
        let backrank = Rank::R1.relative(us);

        let king_square = mv.from();
        let rook_square = mv.to();
        let king_to = Square::new(side.king_to_file(), backrank);
        let rook_to = Square::new(side.rook_to_file(), backrank);

        let rook_file = if let Some(file) = self.castling_rights().get(us, side) {
            file
        } else {
            return Err(MoveError::MissingCastlingRights);
        };

        if rook_square as u8 != Square::new(rook_file, backrank) as u8 {
            return Err(MoveError::InvalidCastlingSquare);
        }

        match self.piece_on(king_square) {
            Some(piece) if piece.color() as u8 == us as u8 => {
                if !matches!(piece.kind(), PieceKind::King) {
                    return Err(MoveError::InvalidMoveKind);
                }
            }
            _ => return Err(MoveError::InvalidMovingPiece),
        }

        let path =
            Bitboard::between(king_square, king_to).union(Bitboard::between(rook_square, rook_to));
        if !self
            .occupied()
            .without(king_square)
            .without(rook_square)
            .intersect(path)
            .is_empty()
        {
            return Err(MoveError::ObstructedCastlingPath);
        }
        Ok(())
    }

    /// Checks if a move is pseudolegal.
    ///
    /// A pseudolegal move is one that follows the movement rules of chess
    /// but may leave the king in check. This function validates:
    /// - The moving piece belongs to the player to move
    /// - The move is valid for the piece type
    /// - Captures have a piece to capture
    /// - En passant is only on the en passant square
    /// - Double push is only from the starting rank with clear path
    /// - Promotions are only on the promotion rank
    /// - Castling rights and path are valid (for castling)
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to validate
    ///
    /// # Errors
    ///
    /// Returns `Err(MoveError::InvalidMovingPiece)` if no piece or wrong color piece on from square
    /// Returns `Err(MoveError::MissingCapturedPiece)` if capture but no piece on to square
    /// Returns `Err(MoveError::PieceCaptured)` if non-capture but piece on to square
    /// Returns `Err(MoveError::InvalidToSquare)` if the destination is not reachable
    /// Returns `Err(MoveError::InvalidEnPassant)` if en passant to wrong square
    /// Returns `Err(MoveError::InvalidDoublePush)` if double push conditions not met
    /// Returns `Err(MoveError::InvalidPawnMove)` if pawn move violates promotion rules
    /// Returns `Err(MoveError::InvalidMoveKind)` if move kind invalid for piece type
    /// Returns castling-specific errors if the move is castling
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// assert!(board.check_pseudolegal_move(mv).is_ok());
    /// ```
    pub const fn check_pseudolegal_move(&self, mv: Move) -> Result<(), MoveError> {
        if mv.kind().is_castling() {
            return self.check_pseudolegal_castling(mv);
        }

        let us = self.side_to_move();
        let occupied = self.occupied();

        let from = mv.from();
        let to = mv.to();
        let kind = mv.kind();
        let moving = match self.piece_on(from) {
            Some(piece) if piece.color() as u8 == us as u8 => piece,
            _ => return Err(MoveError::InvalidMovingPiece),
        };
        let captured = self.piece_on(to);
        if captured.is_none() && kind.is_capture() && !matches!(kind, MoveKind::EnPassant) {
            return Err(MoveError::MissingCapturedPiece);
        }
        if captured.is_some() && !kind.is_capture() {
            return Err(MoveError::PieceCaptured);
        }

        if matches!(moving.kind(), PieceKind::Pawn) {
            #[allow(clippy::collapsible_if)]
            if kind.is_capture() {
                if !pawn_attacks(from, us).get(to) {
                    return Err(MoveError::InvalidToSquare);
                }
            } else if !matches!(kind, MoveKind::DoublePush) {
                if from.forward(us) as u8 != to as u8 {
                    return Err(MoveError::InvalidToSquare);
                }
            }
            if matches!(kind, MoveKind::EnPassant) {
                match self.en_passant() {
                    Some(square) if to as u8 == square as u8 => (),
                    _ => return Err(MoveError::InvalidEnPassant),
                }
            }
            if matches!(kind, MoveKind::DoublePush) {
                if !(matches!(from.rank().relative(us), Rank::R2)
                    && !occupied.get(from.forward(us)))
                {
                    return Err(MoveError::InvalidDoublePush);
                }
                if from.forward(us).forward(us) as u8 != to as u8 {
                    return Err(MoveError::InvalidToSquare);
                }
            }
            if kind.is_promotion() ^ matches!(from.rank().relative(us), Rank::R7) {
                return Err(MoveError::InvalidPawnMove);
            }
            return Ok(());
        } else if matches!(kind, MoveKind::DoublePush | MoveKind::EnPassant) || kind.is_promotion()
        {
            return Err(MoveError::InvalidMoveKind);
        }

        if !piece_attacks(moving, from, occupied).get(to) {
            return Err(MoveError::InvalidToSquare);
        }

        Ok(())
    }

    /// Checks if a move is legal.
    ///
    /// A legal move is a pseudolegal move that does not leave the king in check.
    /// This function validates:
    /// - The move is pseudolegal (debug_assert only)
    /// - The king is not moving into check
    /// - The king is not castling through or out of check
    /// - Pinned pieces do not move out of their pin line
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to validate (must be pseudolegal)
    ///
    /// # Errors
    ///
    /// Returns `Err(MoveError::KingIsAttacked)` if the king would be in check after the move
    /// Returns `Err(MoveError::PinnedPieceMovesOut)` if a pinned piece moves out of its pin line
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// assert!(board.check_legal_move(mv).is_ok());
    /// ```
    pub const fn check_legal_move(&self, mv: Move) -> Result<(), MoveError> {
        debug_assert!(self.check_pseudolegal_move(mv).is_ok());
        let us = self.side_to_move;

        let from = mv.from();
        let to = mv.to();
        let kind = mv.kind();

        if let Some(side) = kind.castling_side() {
            let king_to = Square::new(side.king_to_file(), Rank::R1.relative(us));
            let path = Bitboard::between(from, king_to).with(from).with(king_to);

            if !self.dangers().intersect(path).is_empty() {
                return Err(MoveError::KingIsAttacked);
            } else if self.pinned(us).get(to) {
                return Err(MoveError::PinnedPieceMovesOut);
            } else {
                return Ok(());
            }
        }

        if matches!(self.piece_on(from).unwrap().kind(), PieceKind::King) {
            if self.dangers().get(to) {
                return Err(MoveError::KingIsAttacked);
            } else {
                return Ok(());
            }
        }

        if self.pinned(us).get(from) && !Bitboard::line(from, to).get(self.king_square(us)) {
            return Err(MoveError::PinnedPieceMovesOut);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardBuilder;
    use crate::{CastlingRights, Color, File, Piece};

    fn create_standard_board() -> CoreBoard {
        BoardBuilder::classical().build().unwrap()
    }

    #[test]
    fn test_valid_quiet_move_knight() {
        let board = create_standard_board();
        let mv = Move::new(Square::G1, Square::F3, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_valid_quiet_move_bishop() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::C1, Some(Piece::WhiteBishop))
            .piece(Square::E8, Some(Piece::BlackKing))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // C1 -> E3 is a valid diagonal move, path is clear
        let mv = Move::new(Square::C1, Square::E3, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_moving_piece() {
        let board = create_standard_board();
        let mv = Move::new(Square::E4, Square::E5, MoveKind::Quiet).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidMovingPiece)
        );
    }

    #[test]
    fn test_invalid_to_square_knight() {
        let board = create_standard_board();
        // G1 -> G3 is not a valid knight move and G3 is empty
        let mv = Move::new(Square::G1, Square::G3, MoveKind::Quiet).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidToSquare)
        );
    }

    #[test]
    fn test_valid_capture() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E4, Some(Piece::WhiteKnight))
            .piece(Square::F6, Some(Piece::BlackPawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E4, Square::F6, MoveKind::Capture).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_capture_without_piece() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E4, Some(Piece::WhiteKnight))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E4, Square::F6, MoveKind::Capture).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::MissingCapturedPiece)
        );
    }

    #[test]
    fn test_non_capture_with_piece() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E4, Some(Piece::WhiteKnight))
            .piece(Square::F6, Some(Piece::BlackPawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E4, Square::F6, MoveKind::Quiet).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::PieceCaptured)
        );
    }

    #[test]
    fn test_valid_pawn_single_push() {
        let board = create_standard_board();
        let mv = Move::new(Square::E2, Square::E3, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_valid_pawn_double_push() {
        let board = create_standard_board();
        let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_pawn_double_push_blocked() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E2, Some(Piece::WhitePawn))
            .piece(Square::E3, Some(Piece::WhitePawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidDoublePush)
        );
    }

    #[test]
    fn test_invalid_pawn_double_push_wrong_rank() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E3, Some(Piece::WhitePawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E3, Square::E5, MoveKind::DoublePush).unwrap();
        // Pawn on E3 (rank 3 relative to white) cannot double push
        // Fails at the rank check in double push validation
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidDoublePush)
        );
    }

    #[test]
    fn test_valid_pawn_capture() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E5, Some(Piece::WhitePawn))
            .piece(Square::D6, Some(Piece::BlackPawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E5, Square::D6, MoveKind::Capture).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_pawn_capture_wrong_square() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E5, Some(Piece::WhitePawn))
            .piece(Square::D5, Some(Piece::BlackPawn)) // Piece on D5
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // Try to capture on D5 (backward diagonal) - not a valid pawn attack
        let mv = Move::new(Square::E5, Square::D5, MoveKind::Capture).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidToSquare)
        );
    }

    #[test]
    fn test_valid_en_passant() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E5, Some(Piece::WhitePawn))
            .piece(Square::D5, Some(Piece::BlackPawn))
            .en_passant(Some(Square::D6))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // En passant is a diagonal capture: E5 captures on D6
        let mv = Move::new(Square::E5, Square::D6, MoveKind::EnPassant).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_en_passant_wrong_square() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E5, Some(Piece::WhitePawn))
            .piece(Square::D5, Some(Piece::BlackPawn))
            .en_passant(Some(Square::D6))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // Try en passant to F6 (diagonal attack square) when en_passant is D6
        let mv = Move::new(Square::E5, Square::F6, MoveKind::EnPassant).unwrap();
        // F6 is a valid pawn attack square from E5, but not the en_passant square
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidEnPassant)
        );
    }

    #[test]
    fn test_invalid_en_passant_no_en_passant_square() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E5, Some(Piece::WhitePawn))
            .piece(Square::D5, Some(Piece::BlackPawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E5, Square::D6, MoveKind::EnPassant).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidEnPassant)
        );
    }

    #[test]
    fn test_valid_promotion() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::A8, Some(Piece::BlackKing)) // Move black king to A8
            .piece(Square::E7, Some(Piece::WhitePawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E7, Square::E8, MoveKind::QueenPromotion).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_promotion_wrong_rank() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E6, Some(Piece::WhitePawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E6, Square::E7, MoveKind::QueenPromotion).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidPawnMove)
        );
    }

    #[test]
    fn test_invalid_non_promotion_on_promotion_rank() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::A8, Some(Piece::BlackKing)) // Move black king to A8 (safe)
            .piece(Square::E7, Some(Piece::WhitePawn))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E7, Square::E8, MoveKind::Quiet).unwrap();
        // Pawn on 7th rank must promote, but move is Quiet
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidPawnMove)
        );
    }

    #[test]
    fn test_valid_promo_capture() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E7, Some(Piece::WhitePawn))
            .piece(Square::D8, Some(Piece::BlackRook))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E7, Square::D8, MoveKind::QueenPromoCapture).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_move_kind_for_non_pawn() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E4, Some(Piece::WhiteKnight))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E4, Square::E6, MoveKind::DoublePush).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidMoveKind)
        );
    }

    #[test]
    fn test_valid_kingside_castling() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::H8, Some(Piece::BlackRook))
            .castling_rights(CastlingRights::new(
                Some(File::H), // White kingside rook on H1
                None,          // No white queenside rook
                Some(File::H), // Black kingside rook on H8
                None,          // No black queenside rook
            ))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::H1, MoveKind::KingCastle).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_valid_queenside_castling() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::A1, Some(Piece::WhiteRook))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::A8, Some(Piece::BlackRook))
            .castling_rights(CastlingRights::new(
                None,          // No white kingside rook
                Some(File::A), // White queenside rook on A1
                None,          // No black kingside rook
                Some(File::A), // Black queenside rook on A8
            ))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::A1, MoveKind::QueenCastle).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_castling_missing_rights() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::H8, Some(Piece::BlackRook))
            .castling_rights(CastlingRights::NONE)
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::H1, MoveKind::KingCastle).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::MissingCastlingRights)
        );
    }

    #[test]
    fn test_invalid_castling_wrong_rook_square() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::H8, Some(Piece::BlackRook))
            .castling_rights(CastlingRights::new(
                Some(File::H), // White kingside rook on H1
                None,          // No white queenside rook
                Some(File::H), // Black kingside rook on H8
                None,          // No black queenside rook
            ))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::G1, MoveKind::KingCastle).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidCastlingSquare)
        );
    }

    #[test]
    fn test_invalid_castling_obstructed_path() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .piece(Square::F1, Some(Piece::WhiteBishop))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::H8, Some(Piece::BlackRook))
            .castling_rights(CastlingRights::new(
                Some(File::H), // White kingside rook on H1
                None,          // No white queenside rook
                Some(File::H), // Black kingside rook on H8
                None,          // No black queenside rook
            ))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::H1, MoveKind::KingCastle).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::ObstructedCastlingPath)
        );
    }

    #[test]
    fn test_valid_rook_move() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::A1, Some(Piece::WhiteRook))
            .piece(Square::E8, Some(Piece::BlackKing))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // A1 -> A4 is valid, path is clear
        let mv = Move::new(Square::A1, Square::A4, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_valid_queen_move() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::D1, Some(Piece::WhiteQueen))
            .piece(Square::E8, Some(Piece::BlackKing))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        // D1 -> D4 is valid, path is clear
        let mv = Move::new(Square::D1, Square::D4, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_valid_king_move() {
        let board = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .side_to_move(Color::White)
            .build()
            .unwrap();
        let mv = Move::new(Square::E1, Square::E2, MoveKind::Quiet).unwrap();
        assert!(board.check_pseudolegal_move(mv).is_ok());
    }

    #[test]
    fn test_invalid_king_move_too_far() {
        let board = create_standard_board();
        let mv = Move::new(Square::E1, Square::E3, MoveKind::Quiet).unwrap();
        assert_eq!(
            board.check_pseudolegal_move(mv),
            Err(MoveError::InvalidToSquare)
        );
    }
}
