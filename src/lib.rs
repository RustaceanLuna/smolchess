#![cfg_attr(feature = "nightly", feature(uint_gather_scatter_bits))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(long_running_const_eval)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod macros;

pub(crate) mod arrayvec;
pub mod attacks;
pub mod bitboard;
pub mod board;
pub mod errors;
pub mod mv;
pub mod types;

pub use bitboard::Bitboard;
pub use board::{BoardBuilder, CoreBoard};
pub use mv::{Move, MoveKind, MoveList};
pub use types::{CastlingRights, CastlingSide, Color, File, Piece, PieceKind, Rank, Square};
