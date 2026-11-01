use core::fmt::{self, Debug, Display, Write};
use core::num::NonZeroU16;
use core::str::FromStr;

use super::board::StateInfo;
use super::zobrist::{castling_key, en_passant_key, piece_square_key, turn_key};
use crate::CastlingSide;
use crate::errors::FenError;
use crate::{Bitboard, CastlingRights, Color, CoreBoard, File, Piece, PieceKind, Rank, Square, errors::BoardError};

const fn frc_backrank(mut id: usize) -> [PieceKind; File::COUNT] {
    const fn update_empties(backrank: &[PieceKind; File::COUNT], empties: &mut [usize; File::COUNT]) {
        let mut i = 0;
        let mut count = 0;

        while i < File::COUNT {
            if matches!(backrank[i], PieceKind::Pawn) {
                empties[count] = i;
                count += 1;
            }
            i += 1;
        }
    }

    let mut backrank = [PieceKind::Pawn; File::COUNT];
    let mut empties = [0; File::COUNT];

    let b = id % 16;
    id /= 16;
    backrank[b % 4 * 2 + 1] = PieceKind::Bishop;
    backrank[b / 4 * 2] = PieceKind::Bishop;
    update_empties(&backrank, &mut empties);

    let q = id % 6;
    id /= 6;
    backrank[empties[q]] = PieceKind::Queen;
    update_empties(&backrank, &mut empties);

    #[rustfmt::skip]
    const KNIGHT_PAIRS: [[usize; 2]; 10] = [
        [0, 1], [0, 2], [0, 3], [0, 4],
        [1, 2], [1, 3], [1, 4],
        [2, 3], [2, 4],
        [3, 4],
    ];
    let n = id;
    let pair = KNIGHT_PAIRS[n];
    backrank[empties[pair[0]]] = PieceKind::Knight;
    backrank[empties[pair[1]]] = PieceKind::Knight;
    update_empties(&backrank, &mut empties);

    backrank[empties[0]] = PieceKind::Rook;
    backrank[empties[1]] = PieceKind::King;
    backrank[empties[2]] = PieceKind::Rook;

    backrank
}

/// Builder for constructing chess positions.
///
/// This struct provides a fluent interface for building chess positions
/// from FEN strings, FRC (Chess960) positions, or by manually placing pieces.
///
/// # Examples
///
/// ```
/// use chess::board::BoardBuilder;
///
/// let builder = BoardBuilder::classical();
/// let core = builder.build().unwrap();
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BoardBuilder {
    mailbox: [Option<Piece>; 64],

    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,

    fullmove_number: NonZeroU16,
    halfmove_clock: u8,

    is_frc: bool,
}

impl BoardBuilder {
    /// Creates a new empty board builder.
    ///
    /// # Returns
    ///
    /// Returns a builder with default values:
    /// - Empty board
    /// - White to move
    /// - No castling rights
    /// - No en passant square
    /// - Fullmove number 1
    /// - Halfmove clock 0
    /// - Not FRC
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            mailbox: [None; 64],

            side_to_move: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passant: None,

            fullmove_number: NonZeroU16::new(1).unwrap(),
            halfmove_clock: 0,

            is_frc: false,
        }
    }

    /// Creates a builder for the classical starting position.
    ///
    /// # Returns
    ///
    /// Returns a builder configured with the standard chess starting position.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::classical();
    /// let core = builder.build().unwrap();
    /// ```
    pub const fn classical() -> Self {
        Self::from_frc(518).unwrap().frc(false)
    }

    /// Creates a builder for a FRC (Chess960) position.
    ///
    /// # Arguments
    ///
    /// * `id` - The FRC position ID (0-959)
    ///
    /// # Returns
    ///
    /// Returns `Some(builder)` if the ID is valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::from_frc(0);
    /// assert!(builder.is_some());
    /// ```
    pub const fn from_frc(id: usize) -> Option<Self> {
        Self::dfrc(id, id)
    }

    /// Creates a builder for a double FRC position (different backranks for each color).
    ///
    /// # Arguments
    ///
    /// * `white_id` - The FRC position ID for white (0-959)
    /// * `black_id` - The FRC position ID for black (0-959)
    ///
    /// # Returns
    ///
    /// Returns `Some(builder)` if both IDs are valid, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::dfrc(0, 0);
    /// assert!(builder.is_some());
    /// ```
    pub const fn dfrc(white_id: usize, black_id: usize) -> Option<Self> {
        if white_id >= 960 || black_id >= 960 {
            return None;
        }
        let mut this = Self::new()
            .frc(true)
            .pieces(Rank::R2.to_bitboard(), Some(Piece::WhitePawn))
            .pieces(Rank::R7.to_bitboard(), Some(Piece::BlackPawn));
        let mut castling_rights = CastlingRights::NONE;

        let white_backrank = frc_backrank(white_id);
        let black_backrank = frc_backrank(black_id);
        let mut white_side = CastlingSide::Queenside;
        let mut black_side = CastlingSide::Queenside;

        let mut curr_file = Some(File::A);
        while let Some(file) = curr_file {
            curr_file = file.shift(1);
            let white_piece = white_backrank[file as usize].white();
            let black_piece = black_backrank[file as usize].black();

            this = this
                .piece(Square::new(file, Rank::R1), Some(white_piece))
                .piece(Square::new(file, Rank::R8), Some(black_piece));

            if matches!(white_backrank[file as usize], PieceKind::Rook) {
                castling_rights.set(Color::White, white_side, file);
                white_side = white_side.other();
            }
            if matches!(black_backrank[file as usize], PieceKind::Rook) {
                castling_rights.set(Color::Black, black_side, file);
                black_side = black_side.other();
            }
        }

        this = this.castling_rights(castling_rights);
        Some(this)
    }

    /// Parses a FEN string to create a board builder.
    ///
    /// # Arguments
    ///
    /// * `fen` - The FEN string to parse
    ///
    /// # Returns
    ///
    /// Returns `Ok(builder)` if the FEN is valid, `Err(FenError)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `Err(FenError)` if the FEN string is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    /// assert!(builder.is_ok());
    /// ```
    pub const fn from_fen(fen: &str) -> Result<Self, FenError> {
        const fn next_token(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
            let n = bytes.len();
            let mut i = 0;
            while i < n && bytes[i] == b' ' {
                i += 1;
            }

            let start = i;
            while i < n && bytes[i] != b' ' {
                i += 1;
            }
            if start == i {
                return None;
            }

            let bytes = bytes.split_at(start).1;
            Some(bytes.split_at(i - start))
        }

        const fn parse_u16(bytes: &[u8]) -> Option<u16> {
            if bytes.is_empty() {
                return None;
            }
            let mut value = 0u16;
            let mut i = 0;
            while i < bytes.len() {
                let byte = bytes[i];
                let digit = if byte.is_ascii_digit() {
                    (byte - b'0') as u16
                } else {
                    return None;
                };
                if let Some(x) = value.checked_mul(10) {
                    if let Some(y) = x.checked_add(digit) {
                        value = y;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
                i += 1;
            }

            Some(value)
        }

        if !fen.is_ascii() {
            return Err(FenError::NotAscii);
        }

        let mut bytes = fen.as_bytes();
        let mut count = 0;
        let mut tokens: [&[u8]; 6] = [b""; 6];
        while let Some((token, rest)) = next_token(bytes) {
            if count < 6 {
                tokens[count] = token;
                bytes = rest;
            }
            count += 1;
        }

        if count != 6 {
            return Err(FenError::InvalidTokenCount);
        }

        let mut builder = Self::new();

        let mut file = 0;
        let mut rank = 7;
        let mut i = 0;
        while i < tokens[0].len() {
            let byte = tokens[0][i];
            match byte {
                b'/' => {
                    if file != 8 {
                        return Err(FenError::InvalidRankSize);
                    }
                    if rank == 0 {
                        return Err(FenError::InvalidRankCount);
                    }
                    file = 0;
                    rank -= 1;
                }
                b'1'..=b'8' => {
                    file += byte - b'0';
                    if file > 8 {
                        return Err(FenError::InvalidRankSize);
                    }
                }
                _ => {
                    if file == 8 {
                        return Err(FenError::InvalidRankSize);
                    }
                    let piece = match Piece::from_char(byte as char) {
                        Some(piece) => piece,
                        None => return Err(FenError::InvalidPiece),
                    };
                    let square = Square::new(File::from_u8(file).unwrap(), Rank::from_u8(rank).unwrap());
                    builder = builder.piece(square, Some(piece));
                    file += 1;
                }
            }
            i += 1;
        }
        if file != 8 {
            return Err(FenError::InvalidRankSize);
        }
        if rank != 0 {
            return Err(FenError::InvalidRankCount);
        }

        let king_squares = [
            match builder.king_square(Color::White) {
                Some(square) => square,
                None => return Err(FenError::MissingKing { color: Color::White }),
            },
            match builder.king_square(Color::Black) {
                Some(square) => square,
                None => return Err(FenError::MissingKing { color: Color::Black }),
            },
        ];

        builder = builder.side_to_move(match tokens[1] {
            b"w" => Color::White,
            b"b" => Color::Black,
            _ => return Err(FenError::InvalidColor),
        });

        if !matches!(tokens[2], b"-") {
            let mut i = 0;
            while i < tokens[2].len() {
                let byte = tokens[2][i];
                let color = Color::from_white(byte.is_ascii_uppercase());
                let backrank = Rank::R1.relative(color);
                let king_file = king_squares[color as usize].file();
                let rook = color.rook();

                let file = match byte.to_ascii_lowercase() {
                    b'k' => {
                        let mut file = File::H;
                        while file as u8 != king_file as u8 {
                            if let Some(piece) = builder.piece_on(Square::new(file, backrank))
                                && piece as u8 == rook as u8
                            {
                                break;
                            }
                            file = file.shift(-1).unwrap();
                        }
                        if file as u8 == king_file as u8 { File::H } else { file }
                    }
                    b'q' => {
                        let mut file = File::A;
                        while file as u8 != king_file as u8 {
                            if let Some(piece) = builder.piece_on(Square::new(file, backrank))
                                && piece as u8 == rook as u8
                            {
                                break;
                            }
                            file = file.shift(1).unwrap();
                        }
                        if file as u8 == king_file as u8 { File::A } else { file }
                    }
                    _ => match File::from_char(byte as char) {
                        Some(file) => file,
                        None => return Err(FenError::InvalidCastlingRights),
                    },
                };
                let side = CastlingSide::kingside(file as u8 > king_file as u8);
                builder.castling_rights.set(color, side, file);

                i += 1;
            }
        }

        match tokens[3].len() {
            1 if matches!(tokens[3], b"-") => (),
            2 => match (
                File::from_char(tokens[3][0] as char),
                Rank::from_char(tokens[3][1] as char),
            ) {
                (Some(file), Some(rank)) => builder.en_passant = Some(Square::new(file, rank)),
                _ => return Err(FenError::InvalidEnPassant),
            },
            _ => return Err(FenError::InvalidEnPassant),
        }

        builder.halfmove_clock = match parse_u16(tokens[4]) {
            Some(x) if x <= 150 => x as u8,
            _ => return Err(FenError::InvalidHalfmoveClock),
        };
        builder.fullmove_number = match parse_u16(tokens[5]) {
            Some(x) if x > 0 => NonZeroU16::new(x).unwrap(),
            _ => return Err(FenError::InvalidFullmoveNumber),
        };

        Ok(builder)
    }

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
    /// use chess::{Square, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::classical();
    /// assert!(builder.piece_on(Square::E2).is_some());
    /// ```
    pub const fn piece_on(&self, square: Square) -> Option<Piece> {
        self.mailbox[square as usize]
    }

    /// Returns the square of the king for a color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to find the king for
    ///
    /// # Returns
    ///
    /// Returns `Some(square)` if the king is found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::classical();
    /// assert!(builder.king_square(Color::White).is_some());
    /// ```
    pub const fn king_square(&self, color: Color) -> Option<Square> {
        let king = color.king();
        let mut i = 0;
        while i < Square::COUNT {
            if let Some(piece) = self.mailbox[i]
                && piece as u8 == king as u8
            {
                return Square::from_u8(i as u8);
            }
            i += 1;
        }
        None
    }

    /// Sets a piece on a square.
    ///
    /// # Arguments
    ///
    /// * `square` - The square to place the piece on
    /// * `piece` - The piece to place (None to clear)
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, Piece, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::new().piece(Square::E4, Some(Piece::WhiteQueen));
    /// ```
    pub const fn piece(mut self, square: Square, piece: Option<Piece>) -> Self {
        self.mailbox[square as usize] = piece;
        self
    }

    /// Sets pieces on all squares in a bitboard.
    ///
    /// # Arguments
    ///
    /// * `bitboard` - The squares to set pieces on
    /// * `piece` - The piece to place (None to clear)
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Bitboard, Piece, Rank, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::new().pieces(Rank::R2.to_bitboard(), Some(Piece::WhitePawn));
    /// ```
    pub const fn pieces(mut self, mut bitboard: Bitboard, piece: Option<Piece>) -> Self {
        while let Some(square) = bitboard.pop_lsb() {
            self.mailbox[square as usize] = piece;
        }
        self
    }

    /// Sets the side to move.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to move
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Color, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::new().side_to_move(Color::Black);
    /// ```
    pub const fn side_to_move(mut self, color: Color) -> Self {
        self.side_to_move = color;
        self
    }

    /// Sets the castling rights.
    ///
    /// # Arguments
    ///
    /// * `castling_rights` - The castling rights
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{CastlingRights, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::new().castling_rights(CastlingRights::NONE);
    /// ```
    pub const fn castling_rights(mut self, castling_rights: CastlingRights) -> Self {
        self.castling_rights = castling_rights;
        self
    }

    /// Sets the en passant square.
    ///
    /// # Arguments
    ///
    /// * `square` - The en passant square (None to clear)
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{Square, board::BoardBuilder};
    ///
    /// let builder = BoardBuilder::new().en_passant(Some(Square::E3));
    /// ```
    pub const fn en_passant(mut self, square: Option<Square>) -> Self {
        self.en_passant = square;
        self
    }

    /// Sets the fullmove number.
    ///
    /// # Arguments
    ///
    /// * `fullmoves` - The fullmove number
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::{board::BoardBuilder};
    /// use core::num::NonZeroU16;
    ///
    /// let builder = BoardBuilder::new().fullmove_number(NonZeroU16::new(10).unwrap());
    /// ```
    pub const fn fullmove_number(mut self, fullmoves: NonZeroU16) -> Self {
        self.fullmove_number = fullmoves;
        self
    }

    /// Sets the halfmove clock.
    ///
    /// # Arguments
    ///
    /// * `halfmoves` - The halfmove clock
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::new().halfmove_clock(50);
    /// ```
    pub const fn halfmove_clock(mut self, halfmoves: u8) -> Self {
        self.halfmove_clock = halfmoves;
        self
    }

    /// Sets whether this is an FRC position.
    ///
    /// # Arguments
    ///
    /// * `is_frc` - Whether this is FRC
    ///
    /// # Returns
    ///
    /// Returns the modified builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::new().frc(true);
    /// ```
    pub const fn frc(mut self, is_frc: bool) -> Self {
        self.is_frc = is_frc;
        self
    }

    // CoreBoard::update_check_info() calls CoreBoard::king_square, which may cause UB if unchecked, and build_core is used in build, so board.update_check_info is left to build_unchecked
    const unsafe fn build_core(self) -> CoreBoard {
        let state = StateInfo {
            captured_piece: None,
            castling_rights: self.castling_rights,
            checkers: Bitboard::EMPTY,
            dangers: Bitboard::EMPTY,
            en_passant: self.en_passant,
            halfmove_clock: self.halfmove_clock,
            mv: None,
            pinned: [Bitboard::EMPTY; 2],
            pinners: [Bitboard::EMPTY; 2],
            repetition: 1,
            zobrist_key: 0,
        };
        let mut board = CoreBoard {
            mailbox: [None; 64],
            colors: [Bitboard::EMPTY; 2],
            kinds: [Bitboard::EMPTY; 6],

            side_to_move: self.side_to_move,
            halfmove_number: (self.fullmove_number.get() - 1) * 2 + matches!(self.side_to_move, Color::Black) as u16,
            state,

            is_frc: self.is_frc,
        };

        if matches!(board.side_to_move, Color::Black) {
            board.state.zobrist_key ^= turn_key();
        }
        if let Some(square) = board.state.en_passant {
            board.state.zobrist_key ^= en_passant_key(square);
        }
        board.state.zobrist_key ^= castling_key(board.state.castling_rights);

        let mut curr_square = Some(Square::A1);
        while let Some(square) = curr_square {
            curr_square = square.shift(1);
            if let Some(piece) = self.mailbox[square as usize] {
                unsafe {
                    board.put_piece(piece, square);
                }
                board.state.zobrist_key ^= piece_square_key(piece, square);
            }
        }

        board
    }

    /// Builds a CoreBoard with validation.
    ///
    /// This method validates the position before building the board.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CoreBoard)` if the position is valid, `Err(BoardError)` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `Err(BoardError)` if the position is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::classical();
    /// let core = builder.build().unwrap();
    /// ```
    pub const fn build(self) -> Result<CoreBoard, BoardError> {
        let mut board = unsafe { self.build_core() };

        let us = board.side_to_move;
        let them = us.other();

        let mut color = Color::White;
        loop {
            let pawns = board.piece_bb(color, PieceKind::Pawn);
            let rooks = board.piece_bb(color, PieceKind::Rook);

            if !pawns.intersect(Bitboard::BACKRANKS).is_empty() {
                return Err(BoardError::PawnsOnBackranks { color });
            }

            let king_count = board.piece_bb(color, PieceKind::King).popcount();
            if king_count != 1 {
                return Err(BoardError::InvalidKingCount {
                    color,
                    count: king_count,
                });
            }

            let mut excess = 0;
            let mut i = 0;
            while i < PieceKind::King as usize {
                let kind = PieceKind::ALL[i];
                let count = board.piece_bb(color, kind).popcount();
                let max = [8, 10, 10, 10, 9, 1][kind as usize];
                if count > max {
                    return Err(BoardError::TooManyPieces {
                        piece: Piece::new(color, kind),
                        count,
                        max,
                    });
                }
                if let Some(x) = count.checked_sub([255, 2, 2, 2, 1, 255][kind as usize]) {
                    excess += x;
                }

                i += 1;
            }

            let max = 8 - pawns.popcount();
            if excess > max {
                return Err(BoardError::TooManyPromotedPieces { color, max });
            }

            let kingside = board.castling_rights().get(color, CastlingSide::Kingside);
            let queenside = board.castling_rights().get(color, CastlingSide::Kingside);

            if !Rank::R1.relative(color).to_bitboard().get(board.king_square(color))
                && (kingside.is_some() || queenside.is_some())
            {
                return Err(BoardError::InvalidCastlingKing { color });
            }

            if let Some(file) = kingside
                && !rooks.get(Square::new(file, Rank::R1.relative(color)))
            {
                return Err(BoardError::MissingCastlingRook {
                    color,
                    side: CastlingSide::Kingside,
                });
            }

            if let Some(file) = queenside
                && !rooks.get(Square::new(file, Rank::R1.relative(color)))
            {
                return Err(BoardError::MissingCastlingRook {
                    color,
                    side: CastlingSide::Queenside,
                });
            }

            if matches!(color, Color::Black) {
                break;
            }
            color = color.other();
        }

        if board.is_attacked(board.king_square(them), us) {
            return Err(BoardError::KingCanBeCaptured);
        }
        if let Some(square) = board.en_passant()
            && (square.rank() as u8 != Rank::R6.relative(us) as u8
                || match board.piece_on(square.forward(them)) {
                    Some(piece) => piece as u8 != them.pawn() as u8,
                    None => true,
                })
        {
            return Err(BoardError::InvalidEnPassant { square });
        }

        board.update_check_info();
        board.update_en_passant();

        Ok(board)
    }

    /// Builds a CoreBoard without validation.
    ///
    /// # Safety
    ///
    /// The position must be valid. Undefined behavior may occur if this invariant is violated.
    ///
    /// # Returns
    ///
    /// Returns the built CoreBoard.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::classical();
    /// let core = unsafe { builder.build_unchecked() };
    /// ```
    pub const unsafe fn build_unchecked(self) -> CoreBoard {
        let mut board = unsafe { self.build_core() };
        board.update_check_info();
        board.update_en_passant();

        board
    }

    /// Writes the position as a FEN string.
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
    /// use chess::board::BoardBuilder;
    ///
    /// let builder = BoardBuilder::classical();
    /// let mut s = String::new();
    /// builder.write_fen(&mut s).unwrap();
    /// ```
    pub fn write_fen(&self, w: &mut impl Write) -> fmt::Result {
        let mut empty = 0;
        for &rank in Rank::ALL.iter().rev() {
            for file in File::ALL {
                let square = Square::new(file, rank);
                match self.piece_on(square) {
                    Some(piece) => {
                        if empty != 0 {
                            write!(w, "{}", empty)?;
                            empty = 0;
                        }
                        w.write_char(piece.to_char())?
                    }
                    _ => empty += 1,
                }
            }
            if empty != 0 {
                write!(w, "{}", empty)?;
                empty = 0;
            }
            if rank != Rank::R1 {
                w.write_char('/')?;
            }
        }

        w.write_char(' ')?;
        w.write_char(self.side_to_move.to_char())?;
        w.write_char(' ')?;
        if self.is_frc {
            write!(w, "{:#}", self.castling_rights)?;
        } else {
            write!(w, "{}", self.castling_rights)?;
        }
        w.write_char(' ')?;
        match self.en_passant {
            Some(square) => write!(w, "{}", square)?,
            None => w.write_char('-')?,
        }
        w.write_char(' ')?;
        write!(w, "{}", self.halfmove_clock)?;
        w.write_char(' ')?;
        write!(w, "{}", self.fullmove_number.get())?;

        Ok(())
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        BoardBuilder::new()
    }
}

impl Debug for BoardBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.write_fen(f)
    }
}

impl Display for BoardBuilder {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.write_str("  a b c d e f g h\n")?;
        }
        for &rank in Rank::ALL.iter().rev() {
            if f.alternate() {
                f.write_char(rank.to_char())?;
                f.write_char(' ')?;
            }
            for file in File::ALL {
                let ch = self
                    .piece_on(Square::new(file, rank))
                    .map_or('.', |piece| piece.to_char());
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

impl FromStr for BoardBuilder {
    type Err = FenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_fen(s)
    }
}

impl CoreBoard {
    pub const fn to_builder(&self) -> BoardBuilder {
        let mut builder = BoardBuilder::new();
        builder.mailbox = self.mailbox;
        builder.side_to_move = self.side_to_move;
        builder.castling_rights = self.castling_rights();
        builder.en_passant = self.en_passant();
        builder.fullmove_number = NonZeroU16::new(self.fullmove_number()).unwrap();
        builder.halfmove_clock = self.halfmove_clock();
        builder.is_frc = self.is_frc;

        builder
    }
}

impl Debug for CoreBoard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <BoardBuilder as Debug>::fmt(&self.to_builder(), f)
    }
}

impl Display for CoreBoard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <BoardBuilder as Display>::fmt(&self.to_builder(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::NonZeroU16;

    #[test]
    fn test_builder_new() {
        let builder = BoardBuilder::new();
        assert_eq!(builder.side_to_move, Color::White);
        assert_eq!(builder.castling_rights, CastlingRights::NONE);
        assert_eq!(builder.en_passant, None);
        assert_eq!(builder.fullmove_number, NonZeroU16::new(1).unwrap());
        assert_eq!(builder.halfmove_clock, 0);
        assert!(!builder.is_frc);
    }

    #[test]
    fn test_builder_piece() {
        let builder = BoardBuilder::new().piece(Square::E4, Some(Piece::WhiteQueen));
        assert_eq!(builder.piece_on(Square::E4), Some(Piece::WhiteQueen));
        assert_eq!(builder.piece_on(Square::E5), None);
    }

    #[test]
    fn test_builder_pieces() {
        let builder = BoardBuilder::new().pieces(Bitboard::from_rank(Rank::R2), Some(Piece::WhitePawn));
        assert_eq!(builder.piece_on(Square::A2), Some(Piece::WhitePawn));
        assert_eq!(builder.piece_on(Square::B2), Some(Piece::WhitePawn));
        assert_eq!(builder.piece_on(Square::A3), None);
    }

    #[test]
    fn test_builder_side_to_move() {
        let builder = BoardBuilder::new().side_to_move(Color::Black);
        assert_eq!(builder.side_to_move, Color::Black);
    }

    #[test]
    fn test_builder_castling_rights() {
        let rights = CastlingRights::new(Some(File::H), Some(File::A), Some(File::H), Some(File::A));
        let builder = BoardBuilder::new().castling_rights(rights);
        assert_eq!(builder.castling_rights, rights);
    }

    #[test]
    fn test_builder_en_passant() {
        let builder = BoardBuilder::new().en_passant(Some(Square::E3));
        assert_eq!(builder.en_passant, Some(Square::E3));
    }

    #[test]
    fn test_builder_fullmove_number() {
        let builder = BoardBuilder::new().fullmove_number(NonZeroU16::new(10).unwrap());
        assert_eq!(builder.fullmove_number, NonZeroU16::new(10).unwrap());
    }

    #[test]
    fn test_builder_halfmove_clock() {
        let builder = BoardBuilder::new().halfmove_clock(50);
        assert_eq!(builder.halfmove_clock, 50);
    }

    #[test]
    fn test_builder_frc() {
        let builder = BoardBuilder::new().frc(true);
        assert!(builder.is_frc);
    }

    #[test]
    fn test_from_frc_valid() {
        for id in 0..960 {
            assert!(BoardBuilder::from_frc(id).is_some(), "FRC id {} should be valid", id);
        }
    }

    #[test]
    fn test_from_frc_invalid() {
        assert!(BoardBuilder::from_frc(960).is_none());
        assert!(BoardBuilder::from_frc(1000).is_none());
    }

    #[test]
    fn test_dfrc_valid() {
        let builder = BoardBuilder::dfrc(0, 0);
        assert!(builder.is_some());
    }

    #[test]
    fn test_dfrc_invalid() {
        assert!(BoardBuilder::dfrc(960, 0).is_none());
        assert!(BoardBuilder::dfrc(0, 960).is_none());
        assert!(BoardBuilder::dfrc(1000, 1000).is_none());
    }

    #[test]
    fn test_build_empty_board_fails() {
        let builder = BoardBuilder::new();
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_king_can_be_captured() {
        // Place kings such that black king can be captured by white
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::E7, Some(Piece::WhiteQueen)) // Queen attacks E8
            .side_to_move(Color::White);
        let result = builder.build();
        // Black king on E8 is attacked by white queen on E7
        assert!(matches!(result, Err(BoardError::KingCanBeCaptured)));
    }

    #[test]
    fn test_build_invalid_en_passant_wrong_rank() {
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::D5, Some(Piece::BlackPawn))
            .en_passant(Some(Square::E3)) // Wrong rank for white to move
            .side_to_move(Color::White);
        let result = builder.build();
        assert!(matches!(result, Err(BoardError::InvalidEnPassant { .. })));
    }

    #[test]
    fn test_build_invalid_en_passant_no_pawn() {
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .en_passant(Some(Square::E3)) // No pawn behind
            .side_to_move(Color::White);
        let result = builder.build();
        assert!(matches!(result, Err(BoardError::InvalidEnPassant { .. })));
    }

    #[test]
    fn test_build_pawns_on_backranks() {
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::A1, Some(Piece::WhitePawn)); // Pawn on backrank
        let result = builder.build();
        assert!(matches!(
            result,
            Err(BoardError::PawnsOnBackranks { color: Color::White })
        ));
    }

    #[test]
    fn test_build_too_many_pieces() {
        let mut builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing));

        // Add 10 queens (max is 9)
        for i in 0..10 {
            let file = File::from_u8(i % 8).unwrap();
            let rank = Rank::from_u8(2 + (i / 8)).unwrap();
            builder = builder.piece(Square::new(file, rank), Some(Piece::WhiteQueen));
        }

        let result = builder.build();
        assert!(matches!(result, Err(BoardError::TooManyPieces { .. })));
    }

    #[test]
    fn test_build_too_many_promoted_pieces() {
        let mut builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::A2, Some(Piece::WhitePawn))
            .piece(Square::B2, Some(Piece::WhitePawn));

        // Add 8 queens (excess promoted pieces)
        for i in 0..8 {
            let file = File::from_u8(i).unwrap();
            builder = builder.piece(Square::new(file, Rank::R3), Some(Piece::WhiteQueen));
        }

        let result = builder.build();
        assert!(matches!(result, Err(BoardError::TooManyPromotedPieces { .. })));
    }

    #[test]
    fn test_build_invalid_castling_king() {
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .castling_rights(CastlingRights::new(Some(File::H), None, None, None));
        let result = builder.build();
        // King on E1 is on backrank, so this should be valid
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_missing_castling_rook() {
        let builder = BoardBuilder::new()
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::E8, Some(Piece::BlackKing))
            .castling_rights(CastlingRights::new(Some(File::H), None, None, None)); // No rook on H1
        let result = builder.build();
        assert!(matches!(result, Err(BoardError::MissingCastlingRook { .. })));
    }

    #[test]
    fn test_build_valid_standard_position() {
        let builder = BoardBuilder::new()
            .piece(Square::A1, Some(Piece::WhiteRook))
            .piece(Square::B1, Some(Piece::WhiteKnight))
            .piece(Square::C1, Some(Piece::WhiteBishop))
            .piece(Square::D1, Some(Piece::WhiteQueen))
            .piece(Square::E1, Some(Piece::WhiteKing))
            .piece(Square::F1, Some(Piece::WhiteBishop))
            .piece(Square::G1, Some(Piece::WhiteKnight))
            .piece(Square::H1, Some(Piece::WhiteRook))
            .pieces(Bitboard::from_rank(Rank::R2), Some(Piece::WhitePawn))
            .piece(Square::A8, Some(Piece::BlackRook))
            .piece(Square::B8, Some(Piece::BlackKnight))
            .piece(Square::C8, Some(Piece::BlackBishop))
            .piece(Square::D8, Some(Piece::BlackQueen))
            .piece(Square::E8, Some(Piece::BlackKing))
            .piece(Square::F8, Some(Piece::BlackBishop))
            .piece(Square::G8, Some(Piece::BlackKnight))
            .piece(Square::H8, Some(Piece::BlackRook))
            .pieces(Bitboard::from_rank(Rank::R7), Some(Piece::BlackPawn))
            .castling_rights(CastlingRights::new(
                Some(File::H),
                Some(File::A),
                Some(File::H),
                Some(File::A),
            ))
            .side_to_move(Color::White);

        let result = builder.build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_frc_position() {
        let builder = BoardBuilder::from_frc(0).unwrap();
        let result = builder.build();
        assert!(result.is_ok());
    }
}
