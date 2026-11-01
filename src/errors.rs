use thiserror::Error;
use crate::{CastlingSide, Color, Piece, Square};

/// Errors that can occur when parsing chess strings.
///
/// This enum represents errors that can occur when parsing strings
/// into chess types like colors, files, ranks, squares, and pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("the string doesn't contain a valid color")]
    InvalidColor,
    #[error("the string doesn't contain a valid file")]
    InvalidFile,
    #[error("the string doesn't contain a valid rank")]
    InvalidRank,
    #[error("the string doesn't contain a valid square")]
    InvalidSquare,
    #[error("the string doesn't contain a valid piece kind")]
    InvalidPieceKind,
    #[error("the string doesn't contain a valid piece")]
    InvalidPiece,
}

/// Errors that can occur when parsing FEN strings.
///
/// This enum represents errors that can occur when parsing Forsyth-Edwards Notation
/// strings into chess positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FenError {
    #[error("the FEN string contains non ASCII characters")]
    NotAscii,
    #[error("invalid token count (expected 6)")]
    InvalidTokenCount,
    #[error("found a rank with invalid square count (expected 8)")]
    InvalidRankSize,
    #[error("invalid number of ranks (expected 8)")]
    InvalidRankCount,
    #[error("found a invalid character for a piece")]
    InvalidPiece,
    #[error("invalid token for color")]
    InvalidColor,
    #[error("missing {color} king")]
    MissingKing { color: Color },
    #[error("invalid token for castling rights")]
    InvalidCastlingRights,
    #[error("invalid token for en passant square")]
    InvalidEnPassant,
    #[error("invalid token for halfmove clock (must be an integer between 0 and 150)")]
    InvalidHalfmoveClock,
    #[error("invalid token for fullmove number (must be a positive integer)")]
    InvalidFullmoveNumber,
}

/// Errors that can occur when validating board positions.
///
/// This enum represents errors that can occur when validating a chess position
/// for legality, such as invalid piece counts, missing kings, or invalid castling rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum BoardError {
    #[error("found {count} {color:?} kings (expected 1)")]
    InvalidKingCount { color: Color, count: u8 },
    #[error("found {count} {piece:?} (maximum possible is {max})")]
    TooManyPieces { piece: Piece, count: u8, max: u8 },
    #[error("found too many {color:?} promoted pieces of (maximum possible is {max})")]
    TooManyPromotedPieces { color: Color, max: u8 },
    #[error("found {color:?} pawns on the backranks")]
    PawnsOnBackranks { color: Color },
    #[error("the king of the side not to move can be captured")]
    KingCanBeCaptured,

    #[error("the {color:?} king must be on its backrank to castle")]
    InvalidCastlingKing { color: Color },
    #[error("{color:?} rook for {side:?} castling not found")]
    MissingCastlingRook { color: Color, side: CastlingSide },
    #[error("invalid en passant square: {square:?}")]
    InvalidEnPassant { square: Square },
}

/// Errors that can occur when validating chess moves.
///
/// This enum represents errors that can occur when validating a move for legality,
/// such as invalid piece movement, castling violations, or king safety violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoveError {
    #[error("the piece on from square doesn't contain a piece of the side to move")]
    InvalidMovingPiece,
    #[error("invalid move kind for the moving piece")]
    InvalidMoveKind,
    #[error("capturing move doesn't capture a piece of the side not to move")]
    MissingCapturedPiece,
    #[error("non capturing move captures a piece")]
    PieceCaptured,
    #[error("to square of en passant move is not the en passant square, or there is no en passant square")]
    InvalidEnPassant,
    #[error("the piece on the from square cannot move to the to square")]
    InvalidToSquare,
    #[error("double push from square is not on the relative 2th rank, or the pawn is blocked")]
    InvalidDoublePush,
    #[error("from square must be on the relative 7th rank if and only if the move is a promotion")]
    InvalidPawnMove,

    #[error("castling rights of the castling move's side is not set")]
    MissingCastlingRights,
    #[error("rook square of castling move is not valid")]
    InvalidCastlingSquare,
    #[error("castling path of castling move is obstructed")]
    ObstructedCastlingPath,

    #[error("king moves into or through an attacked square")]
    KingIsAttacked,
    #[error("pinned piece moves out of the pin ray")]
    PinnedPieceMovesOut,
}

/// Error returned when attempting to undo a move with empty history.
///
/// This error occurs when trying to undo a move but the history stack is empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
#[error("cannot undo move because history stack is empty")]
pub struct EmptyHistory;
