use core::fmt::{self, Debug, Display, Write};
use core::ops::{Deref, Index};
use core::slice;

use crate::arrayvec::ArrayVec;
use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, rook_attacks};
use crate::{Bitboard, CastlingSide, Color, CoreBoard, File, Move, MoveKind, PieceKind, Rank, Square};

/// A list of chess moves.
///
/// This struct stores moves in a fixed-capacity array (max 218 moves, which is
/// the theoretical maximum number of legal moves in a chess position).
///
/// # Examples
///
/// ```
/// use chess::{mv::{MoveList, movegen::AllMoves}, board::BoardBuilder};
///
/// let board = BoardBuilder::classical().build().unwrap();
/// let moves = MoveList::new::<AllMoves>(&board);
/// println!("{} moves available", moves.len());
/// ```
#[derive(Clone, PartialEq, Hash)]
pub struct MoveList(ArrayVec<Move, 218>);

impl MoveList {
    /// Returns the number of moves in the list.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::{MoveList, movegen::AllMoves}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = MoveList::new::<AllMoves>(&board);
    /// let count = moves.len();
    /// ```
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::{MoveList, movegen::AllMoves}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = MoveList::new::<AllMoves>(&board);
    /// let is_empty = moves.is_empty();
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the moves as a slice.
    ///
    /// # Returns
    ///
    /// Returns a slice of `Move` items.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::{MoveList, movegen::AllMoves}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = MoveList::new::<AllMoves>(&board);
    /// let slice = moves.as_slice();
    /// ```
    pub const fn as_slice(&self) -> &[Move] {
        self.0.as_slice()
    }

    /// Returns an iterator over the moves.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::{MoveList, movegen::AllMoves}, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = MoveList::new::<AllMoves>(&board);
    /// for mv in moves.iter() {
    ///     println!("{}", mv);
    /// }
    /// ```
    pub fn iter<'a>(&'a self) -> slice::Iter<'a, Move> {
        self.0.iter()
    }

    const unsafe fn push(&mut self, from: Square, to: Square, kind: MoveKind) {
        unsafe {
            self.0.push_unchecked(Move::new_unchecked(from, to, kind));
        }
    }

    const unsafe fn extend_from_bb(&mut self, from: Square, mut bitboard: Bitboard, kind: MoveKind) {
        while let Some(to) = bitboard.pop_lsb() {
            unsafe {
                self.push(from, to, kind);
            }
        }
    }

    const unsafe fn extend_from_dir(&mut self, mut bitboard: Bitboard, direction: i8, kind: MoveKind) {
        while let Some(to) = bitboard.pop_lsb() {
            unsafe {
                self.push(to.shift_unchecked(-direction), to, kind);
            }
        }
    }

    const unsafe fn extend_promo_captures(&mut self, bitboard: Bitboard, direction: i8) {
        unsafe {
            self.extend_from_dir(bitboard, direction, MoveKind::KnightPromoCapture);
            self.extend_from_dir(bitboard, direction, MoveKind::BishopPromoCapture);
            self.extend_from_dir(bitboard, direction, MoveKind::RookPromoCapture);
            self.extend_from_dir(bitboard, direction, MoveKind::QueenPromoCapture);
        }
    }
}

trait Col {
    type Other: Col;
    const COLOR: Color;

    const FORWARD: i8 = match Self::COLOR {
        Color::White => 8,
        Color::Black => -8,
    };
    const FORWARD_LEFT: i8 = Self::FORWARD - 1;
    const FORWARD_RIGHT: i8 = Self::FORWARD + 1;

    const BACKRANK: Rank = Rank::R1.relative(Self::COLOR);

    const RANK_3: Bitboard = Rank::R3.relative(Self::COLOR).to_bitboard();
    const RANK_7: Bitboard = Rank::R7.relative(Self::COLOR).to_bitboard();
    const RANK_8: Bitboard = Rank::R8.relative(Self::COLOR).to_bitboard();
}

struct White;
struct Black;

impl Col for White {
    type Other = Black;
    const COLOR: Color = Color::White;
}

impl Col for Black {
    type Other = White;
    const COLOR: Color = Color::Black;
}

/// Trait for specifying the kind of moves to generate.
///
/// This trait controls which types of moves are included in move generation:
/// - `HAS_QUIETS`: Include non-capturing moves
/// - `HAS_CAPTURES`: Include capturing moves
/// - `HAS_PROMO_PUSHES`: Include pawn promotion pushes
///
/// # Examples
///
/// ```
/// use chess::mv::movegen::{AllMoves, Quiet, Capture, Tactical, MoveGenKind};
///
/// assert!(AllMoves::HAS_QUIETS);
/// assert!(AllMoves::HAS_CAPTURES);
/// assert!(!Quiet::HAS_CAPTURES);
/// assert!(Tactical::HAS_PROMO_PUSHES);
/// ```
pub trait MoveGenKind {
    const HAS_QUIETS: bool;
    const HAS_CAPTURES: bool;
    const HAS_PROMO_PUSHES: bool;

    const ASSERT: () = assert!(Self::HAS_QUIETS || Self::HAS_CAPTURES);
}

/// Generate all legal moves (quiet and captures).
///
/// # Examples
///
/// ```
/// use chess::{mv::MoveList, mv::movegen::AllMoves, board::BoardBuilder};
///
/// let board = BoardBuilder::classical().build().unwrap();
/// let moves = MoveList::new::<AllMoves>(&board);
/// ```
pub struct AllMoves;

/// Generate only quiet (non-capturing) moves.
///
/// # Examples
///
/// ```
/// use chess::{mv::MoveList, mv::movegen::Quiet, board::BoardBuilder};
///
/// let board = BoardBuilder::classical().build().unwrap();
/// let moves = MoveList::new::<Quiet>(&board);
/// ```
pub struct Quiet;

/// Generate only capturing moves.
///
/// # Examples
///
/// ```
/// use chess::{mv::MoveList, mv::movegen::Capture, board::BoardBuilder};
///
/// let board = BoardBuilder::classical().build().unwrap();
/// let moves = MoveList::new::<Capture>(&board);
/// ```
pub struct Capture;

/// Generate tactical moves (captures and promotions).
///
/// # Examples
///
/// ```
/// use chess::{mv::MoveList, mv::movegen::Tactical, board::BoardBuilder};
///
/// let board = BoardBuilder::classical().build().unwrap();
/// let moves = MoveList::new::<Tactical>(&board);
/// ```
pub struct Tactical;

impl MoveGenKind for AllMoves {
    const HAS_QUIETS: bool = true;
    const HAS_CAPTURES: bool = true;
    const HAS_PROMO_PUSHES: bool = true;
}

impl MoveGenKind for Quiet {
    const HAS_QUIETS: bool = true;
    const HAS_CAPTURES: bool = false;
    const HAS_PROMO_PUSHES: bool = false;
}

impl MoveGenKind for Capture {
    const HAS_QUIETS: bool = false;
    const HAS_CAPTURES: bool = true;
    const HAS_PROMO_PUSHES: bool = false;
}

impl MoveGenKind for Tactical {
    const HAS_QUIETS: bool = false;
    const HAS_CAPTURES: bool = true;
    const HAS_PROMO_PUSHES: bool = true;
}

impl MoveList {
    const unsafe fn extend_pawn_pushes<C: Col, M: MoveGenKind>(&mut self, board: &CoreBoard, target: Bitboard) {
        let pawns = board.piece_bb(C::COLOR, PieceKind::Pawn).diff(
            board
                .pinned(C::COLOR)
                .diff(board.king_square(C::COLOR).file().to_bitboard()),
        );
        let empty = board.occupied().flip();

        if M::HAS_QUIETS {
            let pawns = pawns.diff(C::RANK_7);
            let single_pushes = pawns.shift(C::FORWARD).intersect(empty);
            let double_pushes = single_pushes.intersect(C::RANK_3).shift(C::FORWARD).intersect(empty);

            unsafe {
                self.extend_from_dir(single_pushes.intersect(target), C::FORWARD, MoveKind::Quiet);
                self.extend_from_dir(double_pushes.intersect(target), C::FORWARD * 2, MoveKind::DoublePush);
            }
        }

        if M::HAS_QUIETS || M::HAS_PROMO_PUSHES {
            let promo_pushes = pawns
                .intersect(C::RANK_7)
                .shift(C::FORWARD)
                .intersect(empty)
                .intersect(target);
            unsafe {
                self.extend_from_dir(promo_pushes, C::FORWARD, MoveKind::KnightPromotion);
                self.extend_from_dir(promo_pushes, C::FORWARD, MoveKind::BishopPromotion);
                self.extend_from_dir(promo_pushes, C::FORWARD, MoveKind::RookPromotion);
                self.extend_from_dir(promo_pushes, C::FORWARD, MoveKind::QueenPromotion);
            }
        }
    }

    const unsafe fn extend_pawn_captures<C: Col>(&mut self, board: &CoreBoard, target: Bitboard) {
        let king_square = board.king_square(C::COLOR);
        let left_pin = if matches!(C::COLOR, Color::White) {
            Bitboard::anti_diagonal(king_square)
        } else {
            Bitboard::diagonal(king_square)
        };
        let right_pin = if matches!(C::COLOR, Color::White) {
            Bitboard::diagonal(king_square)
        } else {
            Bitboard::anti_diagonal(king_square)
        };

        let target = target.intersect(board.color_bb(C::Other::COLOR));
        let pawns = board.piece_bb(C::COLOR, PieceKind::Pawn);

        let pinned = board.pinned(C::COLOR);
        let left_pawns = pawns.diff(pinned.diff(left_pin)).diff(File::A.to_bitboard());
        let right_pawns = pawns.diff(pinned.diff(right_pin)).diff(File::H.to_bitboard());

        let left_captures = left_pawns.shift(C::FORWARD_LEFT).intersect(target);
        let right_captures = right_pawns.shift(C::FORWARD_RIGHT).intersect(target);

        unsafe {
            self.extend_from_dir(left_captures.diff(C::RANK_8), C::FORWARD_LEFT, MoveKind::Capture);
            self.extend_from_dir(right_captures.diff(C::RANK_8), C::FORWARD_RIGHT, MoveKind::Capture);
        }
        unsafe {
            self.extend_promo_captures(left_captures.intersect(C::RANK_8), C::FORWARD_LEFT);
            self.extend_promo_captures(right_captures.intersect(C::RANK_8), C::FORWARD_RIGHT);
        }
        if let Some(square) = board.en_passant() {
            let bitboard = square.to_bitboard();
            if let Some(pawn) = left_pawns
                .intersect(bitboard.diff(File::H.to_bitboard()).shift(-C::FORWARD_LEFT))
                .pop_lsb()
            {
                unsafe { self.push(pawn, square, MoveKind::EnPassant) }
            }
            if let Some(pawn) = right_pawns
                .intersect(bitboard.diff(File::A.to_bitboard()).shift(-C::FORWARD_RIGHT))
                .pop_lsb()
            {
                unsafe { self.push(pawn, square, MoveKind::EnPassant) }
            }
        }
    }

    const unsafe fn push_castling_side<C: Col>(&mut self, board: &CoreBoard, side: CastlingSide) {
        let king_from = board.king_square(C::COLOR);
        let rook_from = match board.castling_rights().get(C::COLOR, side) {
            Some(file) => Square::new(file, C::BACKRANK),
            None => return,
        };
        let king_to = Square::new(side.king_to_file(), C::BACKRANK);
        let rook_to = Square::new(side.rook_to_file(), C::BACKRANK);

        if !Bitboard::between(king_from, king_to)
            .with(king_from)
            .with(king_to)
            .intersect(board.dangers())
            .is_empty()
        {
            return;
        }

        let path = Bitboard::between(king_from, king_to).union(Bitboard::between(rook_from, rook_to));
        if path
            .without(king_from)
            .without(rook_from)
            .intersect(board.occupied())
            .is_empty()
            && !board.pinned(C::COLOR).get(rook_from)
        {
            unsafe {
                self.push(king_from, rook_from, MoveKind::from_castling(side));
            }
        }
    }

    const unsafe fn extend_castlings<C: Col>(&mut self, board: &CoreBoard) {
        unsafe {
            self.push_castling_side::<C>(board, CastlingSide::Kingside);
            self.push_castling_side::<C>(board, CastlingSide::Queenside);
        }
    }

    const unsafe fn extend_piece_moves<C: Col, M: MoveGenKind>(
        &mut self,
        board: &CoreBoard,
        from: Square,
        bitboard: Bitboard,
    ) {
        unsafe {
            if M::HAS_QUIETS {
                self.extend_from_bb(from, bitboard.diff(board.occupied()), MoveKind::Quiet);
            }
            if M::HAS_CAPTURES {
                self.extend_from_bb(
                    from,
                    bitboard.intersect(board.color_bb(C::Other::COLOR)),
                    MoveKind::Capture,
                );
            }
        }
    }

    const unsafe fn extend_moves_color<C: Col, M: MoveGenKind>(&mut self, board: &CoreBoard) {
        debug_assert!(C::COLOR as u8 == board.side_to_move() as u8);
        let king_square = board.king_square(C::COLOR);
        unsafe {
            self.extend_piece_moves::<C, M>(board, king_square, king_attacks(king_square).diff(board.dangers()));
        }

        let occupied = board.occupied();
        let pinned = board.pinned(C::COLOR);
        let target = match board.checkers().popcount() {
            0 => Bitboard::ALL,
            1 => Bitboard::between(king_square, board.checkers().lsb().unwrap()).union(board.checkers()),
            _ => return,
        };

        unsafe {
            let mut unpinned_pieces;
            let mut pinned_pieces;

            self.extend_pawn_pushes::<C, M>(board, target);
            if M::HAS_CAPTURES {
                self.extend_pawn_captures::<C>(board, target);
            }

            let knights = board.piece_bb(C::COLOR, PieceKind::Knight);
            unpinned_pieces = knights.diff(pinned);
            while let Some(from) = unpinned_pieces.pop_lsb() {
                let attacks = knight_attacks(from).intersect(target);
                self.extend_piece_moves::<C, M>(board, from, attacks);
            }

            let diag_sliders = board.piece2_bb(C::COLOR, PieceKind::Bishop, PieceKind::Queen);
            unpinned_pieces = diag_sliders.diff(pinned);
            pinned_pieces = diag_sliders.intersect(pinned);
            while let Some(from) = unpinned_pieces.pop_lsb() {
                let attacks = bishop_attacks(from, occupied).intersect(target);
                self.extend_piece_moves::<C, M>(board, from, attacks);
            }
            while let Some(from) = pinned_pieces.pop_lsb() {
                let pin_mask = Bitboard::ray_pass(king_square, from);
                let attacks = bishop_attacks(from, occupied).intersect(target).intersect(pin_mask);
                self.extend_piece_moves::<C, M>(board, from, attacks);
            }

            let ortho_sliders = board.piece2_bb(C::COLOR, PieceKind::Rook, PieceKind::Queen);
            unpinned_pieces = ortho_sliders.diff(pinned);
            pinned_pieces = ortho_sliders.intersect(pinned);
            while let Some(from) = unpinned_pieces.pop_lsb() {
                let attacks = rook_attacks(from, occupied).intersect(target);
                self.extend_piece_moves::<C, M>(board, from, attacks);
            }
            while let Some(from) = pinned_pieces.pop_lsb() {
                let pin_mask = Bitboard::ray_pass(king_square, from);
                let attacks = rook_attacks(from, occupied).intersect(target).intersect(pin_mask);
                self.extend_piece_moves::<C, M>(board, from, attacks);
            }

            if M::HAS_QUIETS {
                self.extend_castlings::<C>(board);
            }
        }
    }
}

impl MoveList {
    /// Creates a new move list with all legal moves of the specified kind.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The kind of moves to generate (AllMoves, Quiet, Capture, or Tactical)
    ///
    /// # Arguments
    ///
    /// * `board` - The board to generate moves for
    ///
    /// # Returns
    ///
    /// Returns a MoveList containing the generated moves.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::MoveList, mv::movegen::AllMoves, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = MoveList::new::<AllMoves>(&board);
    /// ```
    pub const fn new<M: MoveGenKind>(board: &CoreBoard) -> Self {
        let mut this = Self(ArrayVec::new());
        unsafe {
            this.extend_moves::<M>(board);
        }
        this
    }

    /// Extends the move list with moves of the specified kind.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The kind of moves to generate (AllMoves, Quiet, Capture, or Tactical)
    ///
    /// # Arguments
    ///
    /// * `board` - The board to generate moves for
    ///
    /// # Safety
    ///
    /// The move list must have enough capacity left to hold all the moves,
    /// Undefined behavior may occur if this invariant is violated.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::MoveList, mv::movegen::Capture, board::BoardBuilder};
    ///
    /// let mut board = BoardBuilder::classical().build().unwrap();
    /// let mut moves = MoveList::new::<Capture>(&board);
    ///
    /// unsafe {
    ///     moves.extend_moves::<Capture>(&board);
    /// }
    /// ```
    pub const unsafe fn extend_moves<M: MoveGenKind>(&mut self, board: &CoreBoard) {
        unsafe {
            match board.side_to_move() {
                Color::White => self.extend_moves_color::<White, M>(board),
                Color::Black => self.extend_moves_color::<Black, M>(board),
            }
        }
    }
}

impl Deref for MoveList {
    type Target = [Move];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type IntoIter = slice::Iter<'a, Move>;
    type Item = &'a Move;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a mut MoveList {
    type IntoIter = slice::IterMut<'a, Move>;
    type Item = &'a mut Move;

    fn into_iter(self) -> Self::IntoIter {
        self.0.as_mut_slice().iter_mut()
    }
}

impl Index<usize> for MoveList {
    type Output = Move;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        for (i, mv) in self.iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            mv.write_uci_frc(f)?;
        }
        f.write_char(']')
    }
}

impl CoreBoard {
    /// Returns all legal moves for the current position.
    ///
    /// This is a convenience function that generates all legal moves (both quiet
    /// and captures) using the `AllMoves` move generation kind.
    ///
    /// # Returns
    ///
    /// Returns a MoveList containing all legal moves.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{mv::MoveList, board::BoardBuilder};
    ///
    /// let board = BoardBuilder::classical().build().unwrap();
    /// let moves = board.legal_moves();
    /// println!("{} legal moves available", moves.len());
    /// ```
    pub const fn legal_moves(&self) -> MoveList {
        MoveList::new::<AllMoves>(self)
    }
}
