use crate::{Bitboard, CoreBoard, PieceKind};

/// Game outcome types.
///
/// This enum represents the possible ways a chess game can end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Checkmate,
    Stalemate,
    InsuffMaterial,

    Repetition3,
    FiftyMoves,

    Repetition5,
    SeventyFiveMoves,
}

impl Outcome {
    /// Checks if the outcome is checkmate.
    ///
    /// # Returns
    ///
    /// Returns true if the outcome is checkmate.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::Outcome;
    ///
    /// assert!(Outcome::Checkmate.is_checkmate());
    /// ```
    pub const fn is_checkmate(self) -> bool {
        matches!(self, Self::Checkmate)
    }

    /// Checks if the outcome is a forced draw.
    ///
    /// Forced draws are draws that must be claimed by the rules:
    /// stalemate, insufficient material, 3-fold repetition, and 75-move rule.
    ///
    /// # Returns
    ///
    /// Returns true if the outcome is a forced draw.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::Outcome;
    ///
    /// assert!(Outcome::Stalemate.is_forced_draw());
    /// ```
    pub const fn is_forced_draw(self) -> bool {
        matches!(
            self,
            Self::Stalemate | Self::InsuffMaterial | Self::Repetition5 | Self::SeventyFiveMoves
        )
    }

    /// Checks if the outcome is a claimable draw.
    ///
    /// Claimable draws are draws that can be claimed by a player:
    /// 3-fold repetition and 50-move rule.
    ///
    /// # Returns
    ///
    /// Returns true if the outcome is a claimable draw.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::Outcome;
    ///
    /// assert!(Outcome::Repetition3.is_claimable_draw());
    /// ```
    pub const fn is_claimable_draw(self) -> bool {
        matches!(self, Self::Repetition3 | Self::FiftyMoves)
    }

    /// Checks if the outcome is any type of draw.
    ///
    /// # Returns
    ///
    /// Returns true if the outcome is a draw (forced or claimable).
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::Outcome;
    ///
    /// assert!(Outcome::Stalemate.is_draw());
    /// ```
    pub const fn is_draw(self) -> bool {
        self.is_claimable_draw() || self.is_forced_draw()
    }
}

impl CoreBoard {
    /// Returns the outcome of the game if it has ended.
    ///
    /// This method checks for checkmate, stalemate, repetition, move count,
    /// and insufficient material.
    ///
    /// # Returns
    ///
    /// Returns `Some(outcome)` if the game has ended, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let outcome = core.outcome();
    /// ```
    pub const fn outcome(&self) -> Option<Outcome> {
        if !self.legal_moves().is_empty() {
            if self.checkers().is_empty() {
                return Some(Outcome::Stalemate);
            } else {
                return Some(Outcome::Checkmate);
            }
        }

        if self.repetition() >= 5 {
            return Some(Outcome::Repetition5);
        }
        if self.halfmove_clock() >= 150 {
            return Some(Outcome::SeventyFiveMoves);
        }

        if self.repetition() >= 3 {
            return Some(Outcome::Repetition3);
        }
        if self.halfmove_clock() >= 100 {
            return Some(Outcome::FiftyMoves);
        }

        if self.kind_bb(PieceKind::Pawn).is_empty()
            && self.kind_bb(PieceKind::Rook).is_empty()
            && self.kind_bb(PieceKind::Queen).is_empty()
        {
            let knights = self.kind_bb(PieceKind::Knight);
            let bishops = self.kind_bb(PieceKind::Bishop);

            if bishops.popcount() == 0 && knights.popcount() < 2 {
                return Some(Outcome::InsuffMaterial);
            }

            if knights.popcount() == 0
                && (bishops.popcount() == 1
                    || (bishops.intersect(Bitboard::LIGHT).is_empty()
                        || bishops.intersect(Bitboard::DARK).is_empty()))
            {
                return Some(Outcome::InsuffMaterial);
            }
        }

        None
    }
}
