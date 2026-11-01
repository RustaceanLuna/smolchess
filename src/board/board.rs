#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::fmt::{self, Debug, Display};
#[cfg(feature = "alloc")]
use core::ops::Deref;

use crate::attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, pawn_attacks_bb, queen_attacks,
    rook_attacks,
};
use crate::{Bitboard, CastlingRights, Color, Move, Piece, PieceKind, Square};

/// State information for a board position.
///
/// This struct contains information about the current state of the board,
/// including castling rights, en passant square, checkers, pins, and zobrist key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateInfo {
    pub zobrist_key: u64,

    pub castling_rights: CastlingRights,
    pub en_passant: Option<Square>,
    pub captured_piece: Option<Piece>,
    pub mv: Option<Move>,

    pub halfmove_clock: u8,
    pub repetition: u8,
    pub dangers: Bitboard,
    pub checkers: Bitboard,
    pub pinned: [Bitboard; 2],
    pub pinners: [Bitboard; 2],
}

/// Core board representation without move history.
///
/// This struct represents a chess position without tracking move history.
/// It provides efficient access to piece positions and attack information.
///
/// # Implementation Note
///
/// `CoreBoard` does not store previous board states. For move history tracking,
/// use the `Board` struct which wraps `CoreBoard` with a history vector.
///
/// # Examples
///
/// ```
/// use smolchess::board::BoardBuilder;
///
/// let core = BoardBuilder::classical().build().unwrap();
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CoreBoard {
    pub(crate) mailbox: [Option<Piece>; 64],
    pub(crate) colors: [Bitboard; 2],
    pub(crate) kinds: [Bitboard; 6],

    pub(crate) side_to_move: Color,
    pub(crate) halfmove_number: u16,
    pub(crate) state: StateInfo,

    pub(crate) is_frc: bool,
}

impl CoreBoard {
    /// Returns the piece on a square.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to check
    ///
    /// # Returns
    ///
    /// Returns `Some(piece)` if a piece is on the square, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// assert!(core.piece_on(Square::E2).is_some());
    /// ```
    pub const fn piece_on(&self, square: Square) -> Option<Piece> {
        self.mailbox[square as usize]
    }

    /// Returns the mailbox array.
    ///
    /// # Returns
    ///
    /// Returns a reference to the mailbox array.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let mailbox = core.mailbox();
    /// ```
    pub const fn mailbox(&self) -> &[Option<Piece>; 64] {
        &self.mailbox
    }

    /// Returns the bitboard of squares occupied by a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to get the bitboard for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares occupied by the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let white = core.color_bb(Color::White);
    /// ```
    pub const fn color_bb(&self, color: Color) -> Bitboard {
        self.colors[color as usize]
    }

    /// Returns the bitboard of squares occupied by a piece kind.
    ///
    /// # Arguments
    ///
    /// * `kind` - The piece kind to get the bitboard for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares occupied by the piece kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{PieceKind, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let pawns = core.kind_bb(PieceKind::Pawn);
    /// ```
    pub const fn kind_bb(&self, kind: PieceKind) -> Bitboard {
        self.kinds[kind as usize]
    }

    /// Returns the bitboard of squares occupied by two piece kinds.
    ///
    /// # Arguments
    ///
    /// * `kind1` - The first piece kind
    /// * `kind2` - The second piece kind
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares occupied by either piece kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{PieceKind, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let sliders = core.kind2_bb(PieceKind::Rook, PieceKind::Queen);
    /// ```
    pub const fn kind2_bb(&self, kind1: PieceKind, kind2: PieceKind) -> Bitboard {
        self.kind_bb(kind1).union(self.kind_bb(kind2))
    }

    /// Returns the bitboard of squares occupied by a color and piece kind.
    ///
    /// # Arguments
    ///
    /// * `color` - The color
    /// * `kind` - The piece kind
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares occupied by the color and piece kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, PieceKind, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let white_pawns = core.piece_bb(Color::White, PieceKind::Pawn);
    /// ```
    pub const fn piece_bb(&self, color: Color, kind: PieceKind) -> Bitboard {
        self.color_bb(color).intersect(self.kind_bb(kind))
    }

    /// Returns the bitboard of squares occupied by a color and two piece kinds.
    ///
    /// # Arguments
    ///
    /// * `color` - The color
    /// * `kind1` - The first piece kind
    /// * `kind2` - The second piece kind
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares occupied by the color and either piece kind.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, PieceKind, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let white_sliders = core.piece2_bb(Color::White, PieceKind::Rook, PieceKind::Queen);
    /// ```
    pub const fn piece2_bb(&self, color: Color, kind1: PieceKind, kind2: PieceKind) -> Bitboard {
        self.color_bb(color).intersect(self.kind2_bb(kind1, kind2))
    }

    /// Returns the bitboard of all occupied squares.
    ///
    /// # Returns
    ///
    /// Returns a bitboard of all occupied squares.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let occupied = core.occupied();
    /// ```
    pub const fn occupied(&self) -> Bitboard {
        self.color_bb(Color::White)
            .union(self.color_bb(Color::Black))
    }

    /// Returns the square of the king for a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to find the king for
    ///
    /// # Returns
    ///
    /// Returns the square of the king.
    ///
    /// # Safety
    ///
    /// The color must have exactly one king on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let king_square = core.king_square(Color::White);
    /// ```
    pub const fn king_square(&self, color: Color) -> Square {
        unsafe {
            self.piece_bb(color, PieceKind::King)
                .pop_lsb()
                .unwrap_unchecked()
        }
    }

    /// Returns the number of pieces of a kind for a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color
    /// * `kind` - The piece kind
    ///
    /// # Returns
    ///
    /// Returns the count of pieces.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, PieceKind, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let count = core.count_pieces(Color::White, PieceKind::Pawn);
    /// assert_eq!(count, 8);
    /// ```
    pub const fn count_pieces(&self, color: Color, kind: PieceKind) -> u8 {
        self.piece_bb(color, kind).popcount()
    }

    /// Returns the side to move.
    ///
    /// # Returns
    ///
    /// Returns the color whose turn it is.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let side = core.side_to_move();
    /// ```
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Returns the halfmove number.
    ///
    /// # Returns
    ///
    /// Returns the number of halfmoves since the start.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let halfmoves = core.halfmove_number();
    /// ```
    pub const fn halfmove_number(&self) -> u16 {
        self.halfmove_number
    }

    /// Returns the fullmove number.
    ///
    /// # Returns
    ///
    /// Returns the number of fullmoves since the start.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let fullmoves = core.fullmove_number();
    /// ```
    pub const fn fullmove_number(&self) -> u16 {
        self.halfmove_number / 2 + 1
    }

    /// Returns whether this is an FRC position.
    ///
    /// # Returns
    ///
    /// Returns true if this is a Chess960/FRC position.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// assert!(!core.is_frc());
    /// ```
    pub const fn is_frc(&self) -> bool {
        self.is_frc
    }

    /// Returns the castling rights.
    ///
    /// # Returns
    ///
    /// Returns the castling rights for the position.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let rights = core.castling_rights();
    /// ```
    pub const fn castling_rights(&self) -> CastlingRights {
        self.state.castling_rights
    }

    /// Returns the en passant square.
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if en passant is available, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let ep = core.en_passant();
    /// ```
    pub const fn en_passant(&self) -> Option<Square> {
        self.state.en_passant
    }

    /// Returns the captured piece.
    ///
    /// # Returns
    ///
    /// Returns `Some(piece)` if a piece was captured, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let captured = core.captured_piece();
    /// ```
    pub const fn captured_piece(&self) -> Option<Piece> {
        self.state.captured_piece
    }

    /// Returns the zobrist key.
    ///
    /// # Returns
    ///
    /// Returns the zobrist hash key for the position.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let key = core.zobrist_key();
    /// ```
    pub const fn zobrist_key(&self) -> u64 {
        self.state.zobrist_key
    }

    /// Returns the halfmove clock.
    ///
    /// # Returns
    ///
    /// Returns the number of halfmoves since the last capture or pawn move.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let clock = core.halfmove_clock();
    /// ```
    pub const fn halfmove_clock(&self) -> u8 {
        self.state.halfmove_clock
    }

    /// Returns the repetition count.
    ///
    /// # Returns
    ///
    /// Returns the number of times this position has occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let repetition = core.repetition();
    /// ```
    pub const fn repetition(&self) -> u8 {
        self.state.repetition
    }

    /// Returns the checkers bitboard.
    ///
    /// # Returns
    ///
    /// Returns a bitboard of pieces giving check to the king.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let checkers = core.checkers();
    /// ```
    pub const fn checkers(&self) -> Bitboard {
        self.state.checkers
    }

    /// Returns the dangers bitboard.
    ///
    /// # Returns
    ///
    /// Returns a bitboard of squares attacked by the opponent.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::BoardBuilder;
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let dangers = core.dangers();
    /// ```
    pub const fn dangers(&self) -> Bitboard {
        self.state.dangers
    }

    /// Returns the pinned pieces bitboard for a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to get pinned pieces for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of pinned pieces for the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let pinned = core.pinned(Color::White);
    /// ```
    pub const fn pinned(&self, color: Color) -> Bitboard {
        self.state.pinned[color as usize]
    }

    /// Returns the pinners bitboard for a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to get pinners for
    ///
    /// # Returns
    ///
    /// Returns a bitboard of pinners for the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Color, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let pinners = core.pinners(Color::Black);
    /// ```
    pub const fn pinners(&self, color: Color) -> Bitboard {
        self.state.pinners[color as usize]
    }

    pub(crate) const unsafe fn put_piece(&mut self, piece: Piece, square: Square) {
        debug_assert!(self.mailbox[square as usize].is_none());
        self.mailbox[square as usize] = Some(piece);
        self.colors[piece.color() as usize].set(square);
        self.kinds[piece.kind() as usize].set(square);
    }

    pub(crate) const unsafe fn remove_piece(&mut self, square: Square) {
        debug_assert!(self.mailbox[square as usize].is_some());
        let piece = unsafe { self.mailbox[square as usize].unwrap_unchecked() };
        self.mailbox[square as usize] = None;
        self.colors[piece.color() as usize].clear(square);
        self.kinds[piece.kind() as usize].clear(square);
    }

    /// Returns the bitboard of pieces attacking a square.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to check
    /// * `occupied` - The occupied squares bitboard
    ///
    /// # Returns
    ///
    /// Returns a bitboard of pieces attacking the square.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, Bitboard, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let attackers = core.attackers_to(Square::E4, core.occupied());
    /// ```
    pub const fn attackers_to(&self, square: Square, occupied: Bitboard) -> Bitboard {
        (self
            .piece_bb(Color::White, PieceKind::Pawn)
            .intersect(pawn_attacks(square, Color::Black)))
        .union(
            self.piece_bb(Color::Black, PieceKind::Pawn)
                .intersect(pawn_attacks(square, Color::White)),
        )
        .union(
            self.kind_bb(PieceKind::Knight)
                .intersect(knight_attacks(square)),
        )
        .union(
            self.kind2_bb(PieceKind::Bishop, PieceKind::Queen)
                .intersect(bishop_attacks(square, occupied)),
        )
        .union(
            self.kind2_bb(PieceKind::Rook, PieceKind::Queen)
                .intersect(rook_attacks(square, occupied)),
        )
        .union(
            self.kind_bb(PieceKind::King)
                .intersect(king_attacks(square)),
        )
    }

    /// Checks if a square is attacked by a color.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to check
    /// * `color` - The attacking color
    ///
    /// # Returns
    ///
    /// Returns true if the square is attacked by the color.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::{Square, Color, board::BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let attacked = core.is_attacked(Square::E4, Color::Black);
    /// ```
    pub const fn is_attacked(&self, square: Square, color: Color) -> bool {
        !(self
            .kind_bb(PieceKind::Pawn)
            .intersect(pawn_attacks(square, color.other())))
        .union(
            self.kind_bb(PieceKind::Knight)
                .intersect(knight_attacks(square)),
        )
        .union(
            self.kind2_bb(PieceKind::Bishop, PieceKind::Queen)
                .intersect(bishop_attacks(square, self.occupied())),
        )
        .union(
            self.kind2_bb(PieceKind::Rook, PieceKind::Queen)
                .intersect(rook_attacks(square, self.occupied())),
        )
        .union(
            self.kind_bb(PieceKind::King)
                .intersect(king_attacks(square)),
        )
        .intersect(self.color_bb(color))
        .is_empty()
    }
}

impl CoreBoard {
    const fn update_pins(&mut self, us: Color) {
        let them = us.other();

        self.state.pinned[us as usize] = Bitboard::EMPTY;
        self.state.pinners[them as usize] = Bitboard::EMPTY;

        let our_king = self.king_square(us);
        let mut sliders = (self
            .piece2_bb(them, PieceKind::Bishop, PieceKind::Queen)
            .intersect(bishop_attacks(our_king, Bitboard::EMPTY)))
        .union(
            self.piece2_bb(them, PieceKind::Rook, PieceKind::Queen)
                .intersect(rook_attacks(our_king, Bitboard::EMPTY)),
        );

        while let Some(slider) = sliders.pop_lsb() {
            let blockers = Bitboard::between(our_king, slider).intersect(self.occupied());
            match blockers.popcount() {
                0 => self.state.checkers.set(slider),
                1 if !self.color_bb(us).intersect(blockers).is_empty() => {
                    self.state.pinned[us as usize].union_assign(blockers);
                    self.state.pinners[them as usize].set(slider);
                }
                _ => (),
            }
        }
    }

    pub(crate) const fn update_check_info(&mut self) {
        let us = self.side_to_move;
        let them = us.other();
        let our_king = self.king_square(us);

        self.state.checkers = (self
            .piece_bb(them, PieceKind::Pawn)
            .intersect(pawn_attacks(our_king, us)))
        .union(
            self.piece_bb(them, PieceKind::Knight)
                .intersect(knight_attacks(our_king)),
        );

        self.update_pins(Color::White);
        self.update_pins(Color::Black);

        let occupied = self.occupied().without(our_king);

        self.state.dangers = pawn_attacks_bb(self.piece_bb(them, PieceKind::Pawn), them);

        let mut knights = self.piece_bb(them, PieceKind::Knight);
        while let Some(square) = knights.pop_lsb() {
            self.state.dangers.union_assign(knight_attacks(square));
        }

        let mut bishops = self.piece_bb(them, PieceKind::Bishop);
        while let Some(square) = bishops.pop_lsb() {
            self.state
                .dangers
                .union_assign(bishop_attacks(square, occupied));
        }

        let mut rooks = self.piece_bb(them, PieceKind::Rook);
        while let Some(square) = rooks.pop_lsb() {
            self.state
                .dangers
                .union_assign(rook_attacks(square, occupied));
        }

        let mut queens = self.piece_bb(them, PieceKind::Queen);
        while let Some(square) = queens.pop_lsb() {
            self.state
                .dangers
                .union_assign(queen_attacks(square, occupied));
        }

        self.state
            .dangers
            .union_assign(king_attacks(self.king_square(them)));
    }

    pub(crate) const fn update_en_passant(&mut self) {
        let us = self.side_to_move;
        let them = us.other();

        let square = match self.en_passant() {
            Some(square) => square,
            None => return,
        };
        let pawn = square.forward(them);
        let king_square = self.king_square(us);

        if !(self.checkers().without(pawn)).is_empty() {
            self.state.en_passant = None;
            return;
        }
        let mut attackers = self
            .piece_bb(us, PieceKind::Pawn)
            .intersect(pawn_attacks(square, them));

        while let Some(attacker) = attackers.pop_lsb() {
            let new_occupied = self.occupied().with(square).without(attacker).without(pawn);
            let checkers = self
                .piece2_bb(them, PieceKind::Bishop, PieceKind::Queen)
                .intersect(bishop_attacks(king_square, new_occupied))
                .union(
                    self.piece2_bb(them, PieceKind::Rook, PieceKind::Queen)
                        .intersect(rook_attacks(king_square, new_occupied)),
                );
            if checkers.is_empty() {
                return;
            }
        }

        self.state.en_passant = None;
    }

    pub const fn update_repetition(&mut self, history: &[StateInfo]) {
        let len = history.len();
        let mut n = self.halfmove_clock() as usize;
        if n > len {
            n = len;
        }
        if n < 4 {
            return;
        }
        let mut i = 4;
        while i <= n {
            let prev = &history[len - i];
            if self.zobrist_key() == prev.zobrist_key {
                self.state.repetition = prev.repetition + 1;
                return;
            }
            i += 2;
        }
    }
}

/// Board representation with move history tracking.
///
/// This struct wraps `CoreBoard` and tracks move history in a vector.
/// It is used for games where move history needs to be preserved.
///
/// # Examples
///
/// ```
/// use smolchess::board::{Board, BoardBuilder};
///
/// let core = BoardBuilder::classical().build().unwrap();
/// let board = Board::new(core);
/// ```
#[cfg(feature = "alloc")]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board {
    pub(crate) core: CoreBoard,
    pub(crate) history: Vec<StateInfo>,
}

#[cfg(feature = "alloc")]
impl Board {
    /// Creates a new Board from a CoreBoard.
    ///
    /// # Arguments
    ///
    /// * `core` - The core board to wrap
    ///
    /// # Returns
    ///
    /// Returns a new Board with empty history.
    ///
    /// # Examples
    ///
    /// ```
    /// use smolchess::board::{Board, BoardBuilder};
    ///
    /// let core = BoardBuilder::classical().build().unwrap();
    /// let board = Board::new(core);
    /// ```
    pub const fn new(core: CoreBoard) -> Self {
        Self {
            core,
            history: Vec::new(),
        }
    }
}

#[cfg(feature = "alloc")]
impl Deref for Board {
    type Target = CoreBoard;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

#[cfg(feature = "alloc")]
impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <CoreBoard as Display>::fmt(&self.core, f)
    }
}
