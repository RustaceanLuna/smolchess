#[cfg(feature = "alloc")]
use crate::Board;
use crate::arrayvec::ArrayVec;
use crate::board::StateInfo;
use crate::board::zobrist::{castling_key, en_passant_key, piece_square_key, turn_key};
#[cfg(feature = "alloc")]
use crate::errors::EmptyHistory;
use crate::errors::MoveError;
use crate::{
    CastlingRights, CastlingSide, Color, CoreBoard, Move, MoveKind, Piece, PieceKind, Rank, Square,
};

/// Buffer for tracking piece changes during move making.
///
/// This buffer stores the pieces that are added (put) and removed from the board
/// during a move operation. This is useful for updating external data structures
/// or for move undo operations.
///
/// # Examples
///
/// ```
/// use chess::mv::PieceChangesBuffer;
///
/// let buffer = PieceChangesBuffer::new();
/// assert!(buffer.put().is_empty());
/// assert!(buffer.remove().is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PieceChangesBuffer {
    put: ArrayVec<(Piece, Square), 2>,
    remove: ArrayVec<(Piece, Square), 2>,
}

impl PieceChangesBuffer {
    /// Creates a new empty buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::mv::PieceChangesBuffer;
    ///
    /// let buffer = PieceChangesBuffer::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            put: ArrayVec::new(),
            remove: ArrayVec::new(),
        }
    }

    /// Clears all piece changes from the buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::mv::PieceChangesBuffer;
    ///
    /// let mut buffer = PieceChangesBuffer::new();
    /// buffer.clear();
    /// ```
    pub const fn clear(&mut self) {
        self.put.clear();
        self.remove.clear();
    }

    /// Returns the pieces that were added to the board.
    ///
    /// # Returns
    ///
    /// Returns a slice of (Piece, Square) tuples representing pieces added.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::mv::PieceChangesBuffer;
    ///
    /// let buffer = PieceChangesBuffer::new();
    /// assert!(buffer.put().is_empty());
    /// ```
    pub const fn put(&self) -> &[(Piece, Square)] {
        self.put.as_slice()
    }

    /// Returns the pieces that were removed from the board.
    ///
    /// # Returns
    ///
    /// Returns a slice of (Piece, Square) tuples representing pieces removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::mv::PieceChangesBuffer;
    ///
    /// let buffer = PieceChangesBuffer::new();
    /// assert!(buffer.remove().is_empty());
    /// ```
    pub const fn remove(&self) -> &[(Piece, Square)] {
        self.remove.as_slice()
    }
}

impl Default for PieceChangesBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreBoard {
    /// Makes a move on the board, tracking piece changes in the buffer.
    ///
    /// This is the base move-making function that handles all move types including
    /// captures, promotions, en passant, and castling. It updates the board state,
    /// zobrist key, castling rights, and en passant square.
    ///
    /// # Safety
    ///
    /// The move must be both pseudolegal and legal. Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    /// * `buffer` - Buffer to track piece changes
    ///
    /// # Returns
    ///
    /// Returns the previous state info for undo purposes.
    pub const unsafe fn make_move_base(
        &mut self,
        mv: Move,
        buffer: &mut PieceChangesBuffer,
    ) -> StateInfo {
        debug_assert!(self.check_pseudolegal_move(mv).is_ok());
        debug_assert!(self.check_legal_move(mv).is_ok());

        let us = self.side_to_move();
        let them = us.other();
        let prev_state = self.state;

        let from = mv.from();
        let to = mv.to();
        let kind = mv.kind();
        let moving = unsafe { self.piece_on(from).unwrap_unchecked() };
        let captured = self.piece_on(to);

        self.state.zobrist_key ^= turn_key() ^ castling_key(self.castling_rights());

        if let Some(square) = self.en_passant() {
            self.state.en_passant = None;
            self.state.zobrist_key ^= en_passant_key(square);
        }

        if kind.is_capture() || matches!(moving.kind(), PieceKind::Pawn) {
            self.state.halfmove_clock = 0;
        } else {
            self.state.halfmove_clock += 1;
        }

        if !kind.is_castling() {
            self.state.captured_piece = captured;
            if let Some(captured) = captured {
                unsafe {
                    self.remove_piece(to);
                }
                self.state.zobrist_key ^= piece_square_key(captured, to);
                buffer.remove.push((captured, to));
            }

            unsafe {
                self.remove_piece(from);
            }
            self.state.zobrist_key ^= piece_square_key(moving, from);
            buffer.put.push((moving, to));

            if !kind.is_promotion() {
                unsafe {
                    self.put_piece(moving, to);
                }
                self.state.zobrist_key ^= piece_square_key(moving, to);
                buffer.remove.push((moving, from));
            }
        }

        #[inline]
        const fn is_cleared(
            color: Color,
            side: CastlingSide,
            from: Square,
            moving: Piece,
            castling_rights: CastlingRights,
        ) -> bool {
            if matches!(moving.kind(), PieceKind::King) {
                return true;
            }
            if let Some(file) = castling_rights.get(color, side) {
                return (matches!(moving.kind(), PieceKind::Rook)
                    && matches!(from.rank().relative(color), Rank::R1)
                    && from.file() as u8 == file as u8);
            }
            false
        }

        if is_cleared(
            us,
            CastlingSide::Kingside,
            from,
            moving,
            self.state.castling_rights,
        ) {
            self.state.castling_rights.clear(us, CastlingSide::Kingside);
        }
        if is_cleared(
            us,
            CastlingSide::Queenside,
            from,
            moving,
            self.state.castling_rights,
        ) {
            self.state
                .castling_rights
                .clear(us, CastlingSide::Queenside);
        }
        if matches!(to.rank(), Rank::R1) {
            self.state
                .castling_rights
                .clear_file(Color::White, to.file());
        } else if matches!(to.rank(), Rank::R8) {
            self.state
                .castling_rights
                .clear_file(Color::Black, to.file());
        }

        match mv.kind() {
            MoveKind::DoublePush => {
                let en_passant = from.forward(us);
                self.state.en_passant = Some(en_passant);
            }
            MoveKind::EnPassant => {
                let captured_square = to.forward(them);
                let their_pawn = them.pawn();
                self.state.captured_piece = Some(their_pawn);

                unsafe {
                    self.remove_piece(captured_square);
                    self.state.zobrist_key ^= piece_square_key(their_pawn, captured_square);
                    buffer.remove.push((their_pawn, captured_square));
                }
            }
            _ => {
                if let Some(side) = kind.castling_side() {
                    let backrank = Rank::R1.relative(us);
                    let our_king = moving;
                    let our_rook = us.rook();
                    let king_from = from;
                    let king_to = Square::new(side.king_to_file(), backrank);
                    let rook_from = to;
                    let rook_to = Square::new(side.rook_to_file(), backrank);

                    unsafe {
                        self.remove_piece(king_from);
                        self.remove_piece(rook_from);
                        self.put_piece(our_king, king_to);
                        self.put_piece(our_rook, rook_to);
                    }
                    self.state.zobrist_key ^= piece_square_key(our_king, king_from)
                        ^ piece_square_key(our_king, king_to)
                        ^ piece_square_key(our_rook, rook_from)
                        ^ piece_square_key(our_rook, rook_to);
                    buffer.remove.push((our_king, king_from));
                    buffer.remove.push((our_rook, rook_from));
                    buffer.put.push((our_king, king_to));
                    buffer.put.push((our_rook, rook_to));
                } else if let Some(piece_kind) = kind.promotion() {
                    let promoted = Piece::new(us, piece_kind);
                    unsafe {
                        self.put_piece(promoted, to);
                    }
                    self.state.zobrist_key ^= piece_square_key(promoted, to);
                    buffer.put.push((promoted, to));
                }
            }
        }

        self.side_to_move.flip();
        self.halfmove_number += 1;

        self.update_check_info();
        self.update_en_passant();

        self.state.zobrist_key ^= castling_key(self.state.castling_rights);
        if let Some(square) = self.en_passant() {
            self.state.zobrist_key ^= en_passant_key(square);
        }
        self.state.mv = Some(mv);

        prev_state
    }

    /// Undoes a move on the board, restoring the previous state.
    ///
    /// This function reverses the effects of a move, restoring pieces to their
    /// original positions and restoring the board state.
    ///
    /// # Safety
    ///
    /// The `prev_state` must be the state returned by the previous `make_move_base`
    /// call. Undefined behavior may occur if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `prev_state` - The previous state info to restore
    /// * `buffer` - Buffer to track piece changes during undo
    pub const unsafe fn undo_move_base(
        &mut self,
        prev_state: &StateInfo,
        buffer: &mut PieceChangesBuffer,
    ) {
        self.side_to_move.flip();
        self.halfmove_number -= 1;

        let us = self.side_to_move;
        let them = us.other();

        let mv = unsafe { self.state.mv.unwrap_unchecked() };
        let from = mv.from();
        let to = mv.to();
        let kind = mv.kind();
        let captured = self.state.captured_piece;

        if !kind.is_castling() {
            let moving = unsafe { self.piece_on(to).unwrap_unchecked() };
            unsafe {
                self.remove_piece(to);
            }
            buffer.remove.push((moving, to));
            if !kind.is_promotion() {
                unsafe {
                    self.put_piece(moving, from);
                }
                buffer.put.push((moving, from));
            }

            if let Some(piece) = captured
                && !matches!(kind, MoveKind::EnPassant)
            {
                unsafe {
                    self.put_piece(piece, to);
                }
                buffer.put.push((piece, to));
            }
        }

        match kind {
            MoveKind::EnPassant => {
                let captured_square = to.forward(them);
                let their_pawn = them.pawn();

                unsafe {
                    self.put_piece(their_pawn, captured_square);
                }
                buffer.put.push((their_pawn, captured_square));
            }
            _ => {
                if let Some(side) = kind.castling_side() {
                    let backrank = Rank::R1.relative(us);
                    let our_king = us.king();
                    let our_rook = us.rook();
                    let king_from = from;
                    let king_to = Square::new(side.king_to_file(), backrank);
                    let rook_from = to;
                    let rook_to = Square::new(side.rook_to_file(), backrank);

                    unsafe {
                        self.remove_piece(king_to);
                        self.remove_piece(rook_to);
                        self.put_piece(our_king, king_from);
                        self.put_piece(our_rook, rook_from);
                    }
                    buffer.remove.push((our_king, king_to));
                    buffer.remove.push((our_rook, rook_to));
                    buffer.put.push((our_king, king_from));
                    buffer.put.push((our_rook, rook_from));
                } else if kind.is_promotion() {
                    let our_pawn = us.pawn();
                    unsafe {
                        self.put_piece(our_pawn, from);
                    }
                    buffer.put.push((our_pawn, from));
                }
            }
        }

        self.state = *prev_state;

        debug_assert!(self.check_pseudolegal_move(mv).is_ok());
        debug_assert!(self.check_legal_move(mv).is_ok());
    }

    /// Makes a move on the board without validation.
    ///
    /// This is a convenience function that calls `make_move_base` with an empty buffer.
    ///
    /// # Safety
    ///
    /// The move must be both pseudolegal and legal. Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    ///
    /// # Returns
    ///
    /// Returns the previous state info for undo purposes.
    pub const unsafe fn make_move_unchecked(&mut self, mv: Move) -> StateInfo {
        unsafe { self.make_move_base(mv, &mut PieceChangesBuffer::new()) }
    }

    /// Makes a move on the board with validation.
    ///
    /// This function validates that the move is both pseudolegal and legal before
    /// making it. This is the safe version of `make_move_unchecked`.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    ///
    /// # Errors
    ///
    /// Returns `Err(MoveError)` if the move is not pseudolegal or not legal.
    pub const fn make_move(&mut self, mv: Move) -> Result<StateInfo, MoveError> {
        if let Err(err) = self.check_pseudolegal_move(mv) {
            return Err(err);
        }
        if let Err(err) = self.check_legal_move(mv) {
            return Err(err);
        }
        unsafe { Ok(self.make_move_unchecked(mv)) }
    }

    /// Undoes a move on the board without validation.
    ///
    /// This is a convenience function that calls `undo_move_base` with an empty buffer.
    ///
    /// # Safety
    ///
    /// The `prev_state` must be the state returned by the previous `make_move`
    /// call. Undefined behavior may occur if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `prev_state` - The previous state info to restore
    pub const unsafe fn undo_move(&mut self, prev_state: &StateInfo) {
        unsafe {
            self.undo_move_base(prev_state, &mut PieceChangesBuffer::new());
        }
    }
}

#[cfg(feature = "alloc")]
use crate::Board;

#[cfg(feature = "alloc")]
impl Board {
    /// Makes a move on the board, tracking piece changes in the buffer.
    ///
    /// This is the Board version of `CoreBoard::make_move_base`, which also pushes
    /// the state to the history and updates repetition detection.
    ///
    /// # Safety
    ///
    /// The move must be both pseudolegal and legal. Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    /// * `buffer` - Buffer to track piece changes
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind, PieceChangesBuffer}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mut buffer = PieceChangesBuffer::new();
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// unsafe {
    ///     board.make_move_base(mv, &mut buffer);
    /// }
    /// ```
    pub unsafe fn make_move_base(&mut self, mv: Move, buffer: &mut PieceChangesBuffer) {
        unsafe {
            self.history.push(self.core.make_move_base(mv, buffer));
        }
        self.core.update_repetition(&self.history);
    }

    /// Makes a move on the board without validation.
    ///
    /// This is a convenience function that calls `make_move_base` with an empty buffer.
    ///
    /// # Safety
    ///
    /// The move must be both pseudolegal and legal. Undefined behavior may occur
    /// if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// unsafe {
    ///     board.make_move_unchecked(mv);
    /// }
    /// ```
    pub unsafe fn make_move_unchecked(&mut self, mv: Move) {
        unsafe {
            self.make_move_base(mv, &mut PieceChangesBuffer::new());
        }
    }

    /// Makes a move on the board with validation.
    ///
    /// This function validates that the move is both pseudolegal and legal before
    /// making it. This is the safe version of `make_move_unchecked`.
    ///
    /// # Arguments
    ///
    /// * `mv` - The move to make
    ///
    /// # Errors
    ///
    /// Returns `Err(MoveError)` if the move is not pseudolegal or not legal.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// let result = board.make_move(mv);
    /// assert!(result.is_ok());
    /// ```
    pub fn make_move(&mut self, mv: Move) -> Result<(), MoveError> {
        self.check_pseudolegal_move(mv)?;
        self.check_legal_move(mv)?;

        unsafe {
            self.make_move_unchecked(mv);
        }
        Ok(())
    }

    /// Undoes a move on the board, restoring the previous state.
    ///
    /// This is the Board version of `CoreBoard::undo_move_base`, which also pops
    /// the state from the history.
    ///
    /// # Safety
    ///
    /// This must be called after a corresponding `make_move_base` call.
    /// Undefined behavior may occur if this invariant is violated.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to track piece changes during undo
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind, PieceChangesBuffer}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mut buffer = PieceChangesBuffer::new();
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// unsafe {
    ///     board.make_move_base(mv, &mut buffer);
    ///     board.undo_move_base(&mut buffer);
    /// }
    /// ```
    pub unsafe fn undo_move_base(&mut self, buffer: &mut PieceChangesBuffer) {
        unsafe {
            self.core
                .undo_move_base(&self.history.pop().unwrap_unchecked(), buffer);
        }
    }

    /// Undoes a move on the board without validation.
    ///
    /// This is a convenience function that calls `undo_move_base` with an empty buffer.
    ///
    /// # Safety
    ///
    /// This must be called after a corresponding `make_move` call.
    /// Undefined behavior may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// unsafe {
    ///     board.make_move_unchecked(mv);
    ///     board.undo_move_unchecked();
    /// }
    /// ```
    pub unsafe fn undo_move_unchecked(&mut self) {
        unsafe {
            self.undo_move_base(&mut PieceChangesBuffer::new());
        }
    }

    /// Undoes a move on the board with validation.
    ///
    /// This function checks that there is a move to undo before undoing it.
    ///
    /// # Errors
    ///
    /// Returns `Err(EmptyHistory)` if there are no moves to undo.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, mv::{Move, MoveKind}, board::{Board, BoardBuilder}};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mut board = Board::new(core);
    /// let mv = Move::new(Square::E2, Square::E4, MoveKind::DoublePush).unwrap();
    ///
    /// board.make_move(mv).unwrap();
    /// let result = board.undo_move();
    /// assert!(result.is_ok());
    /// ```
    pub fn undo_move(&mut self) -> Result<(), EmptyHistory> {
        if self.history.is_empty() {
            return Err(EmptyHistory);
        }
        unsafe {
            self.undo_move_unchecked();
        }
        Ok(())
    }
}
