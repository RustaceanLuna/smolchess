use crate::{Bitboard, CastlingSide, CoreBoard, Move, PieceKind, Square};
use core::fmt::{self, Debug, Display, Write};

impl Move {
    /// Writes the move in UCI notation to the given writer.
    ///
    /// For standard chess, castling moves are written as the king's destination
    /// (e.g., "e1g1" for kingside castling). For Chess960/FRC, the actual rook
    /// square is used.
    ///
    /// # Arguments
    ///
    /// * `w` - The writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// let mut s = String::new();
    /// mv.write_uci(&mut s).unwrap();
    /// assert_eq!(s, "e2e4");
    /// ```
    pub fn write_uci(self, w: &mut impl Write) -> fmt::Result {
        write!(w, "{}", self.from())?;
        let to = self.kind().castling_side().map_or_else(
            || self.to(),
            |side| Square::new(side.king_to_file(), self.to().rank()),
        );
        write!(w, "{}", to)?;
        if let Some(piece_kind) = self.kind().promotion() {
            write!(w, "{}", piece_kind.to_lower_char())?;
        }
        Ok(())
    }

    /// Writes the move in FRC (Chess960) UCI notation to the given writer.
    ///
    /// Unlike standard UCI, FRC notation uses the actual rook square for castling.
    ///
    /// # Arguments
    ///
    /// * `w` - The writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing to the writer fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}};
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// let mut s = String::new();
    /// mv.write_uci_frc(&mut s).unwrap();
    /// assert_eq!(s, "e2e4");
    /// ```
    pub fn write_uci_frc(self, w: &mut impl Write) -> fmt::Result {
        write!(w, "{}{}", self.from(), self.to())?;
        if let Some(piece_kind) = self.kind().promotion() {
            write!(w, "{}", piece_kind.to_lower_char())?;
        }
        Ok(())
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field("kind", &self.kind())
            .finish()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_uci(f)
    }
}

/// A wrapper for displaying a move in Standard Algebraic Notation (SAN).
///
/// This struct is created by `CoreBoard::san` and implements `Display`
/// to format the move according to standard chess notation rules.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, mv::{Move, MoveKind}, board::BoardBuilder};
///
/// let core = BoardBuilder::classical().build().unwrap();
/// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
/// if let Some(san) = core.san(mv) {
///     println!("{}", san);
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct San<'a> {
    mv: Move,
    board: &'a CoreBoard,
}

impl CoreBoard {
    /// Returns a SAN formatter for a move if it is legal.
    ///
    /// This method checks if the move is both pseudolegal and legal before
    /// returning a `San` formatter. If the move is illegal, returns `None`.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to format
    ///
    /// # Returns
    ///
    /// Returns `Some(San)` if the move is legal, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, mv::{Move, MoveKind}, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    /// let san = core.san(mv);
    /// assert!(san.is_some());
    /// ```
    pub const fn san(&self, mv: Move) -> Option<San<'_>> {
        if self.check_pseudolegal_move(mv).is_err() || self.check_legal_move(mv).is_err() {
            return None;
        }
        Some(San { mv, board: self })
    }
}

impl Display for San<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.mv.from();
        let to = self.mv.to();
        let kind = self.mv.kind();
        let moving = self.board.piece_on(from).unwrap();

        match kind.castling_side() {
            Some(CastlingSide::Kingside) => return f.write_str("O-O"),
            Some(CastlingSide::Queenside) => return f.write_str("O-O-O"),
            None => (),
        }

        match moving.kind() {
            PieceKind::Pawn => {
                if kind.is_capture() {
                    f.write_char(from.file().to_char())?;
                }
            }
            _ => f.write_char(moving.kind().to_upper_char())?,
        };

        let move_list = self.board.legal_moves();
        let mut attackers = Bitboard::EMPTY;
        for &mv in &move_list {
            if mv.to() == to && self.board.piece_on(mv.from()) == Some(moving) {
                attackers.set(mv.from());
            }
        }

        let file_ambiguation = (attackers & from.file().to_bitboard()).popcount() != 1;
        let rank_ambiguation = (attackers & from.rank().to_bitboard()).popcount() != 1;
        if (file_ambiguation || rank_ambiguation) && moving.kind() != PieceKind::Pawn {
            if file_ambiguation {
                if rank_ambiguation {
                    f.write_char(from.file().to_char())?;
                }
                f.write_char(from.rank().to_char())?;
            } else {
                f.write_char(from.file().to_char())?;
            }
        }

        if kind.is_capture() {
            f.write_char('x')?;
        }
        write!(f, "{}", to)?;
        if let Some(piece_kind) = kind.promotion() {
            f.write_char('=')?;
            f.write_char(piece_kind.to_upper_char())?;
        }

        unsafe {
            let mut temp = self.board.clone();
            temp.make_move_unchecked(self.mv);
            if !temp.checkers().is_empty() {
                f.write_char(if temp.legal_moves().is_empty() {
                    '#'
                } else {
                    '+'
                })?;
            }
        };

        Ok(())
    }
}
