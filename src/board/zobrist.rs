use crate::{CastlingRights, Piece, Square};

const SEED: u64 = 0x1a7ead09;

const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);

    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

const fn random_arr<const N: usize>(state: &mut u64) -> [u64; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = splitmix64(state);
        i += 1;
    }
    arr
}

const KEYS: ([[u64; 64]; 14], u64, [u64; 16], [u64; 8]) = {
    let mut state = SEED;

    let mut piece_square_keys = [[0; 64]; 14];
    let mut i = 0;
    while i < 14 {
        piece_square_keys[i] = random_arr(&mut state);
        i += 1;
    }

    (
        piece_square_keys,
        splitmix64(&mut state),
        random_arr(&mut state),
        random_arr(&mut state),
    )
};

pub const fn piece_square_key(piece: Piece, square: Square) -> u64 {
    KEYS.0[piece as usize][square as usize]
}

pub const fn turn_key() -> u64 {
    KEYS.1
}

pub const fn castling_key(castling_rights: CastlingRights) -> u64 {
    KEYS.2[castling_rights.bitflags() as usize]
}

pub const fn en_passant_key(square: Square) -> u64 {
    KEYS.3[square.file() as usize]
}
