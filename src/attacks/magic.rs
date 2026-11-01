use super::sliding_attacks;
use crate::{Bitboard, File, PieceKind, Rank, Square};
use core::hint::assert_unchecked;

#[cfg(not(feature = "pext"))]
#[rustfmt::skip]
const BISHOP_MAGICS: [u64; Square::COUNT] = [
    0x0040100100459180, 0xc220480080918180, 0x2408084140804440, 0x0088184102812200,
    0x0082021081010800, 0x0000c42020000024, 0x0010420210400410, 0x0488804800900800,
    0x0814c00474244040, 0x020206528c040280, 0x0000080181020088, 0x0004082041422000,
    0x00000c104c000001, 0x0100420144209880, 0x0230290088200800, 0x0908060901081300,
    0x0085404044082a00, 0x00cc0228908408c0, 0x0411009001002500, 0x1004202806002020,
    0x1100808400a01070, 0x000a020102420201, 0x0000400108080421, 0x0004280a82011010,
    0x021248c020200400, 0x1810306018210100, 0x0408300088054040, 0x1400818108020002,
    0x0220940000806009, 0x8800410206010110, 0x0008005002060a10, 0x1009102052048400,
    0x6202200c01202800, 0x18421030200c2100, 0x8802004040040100, 0x2009020080080280,
    0x0821100400008060, 0x2000848200090504, 0x02900281000a0910, 0x8002342100c02080,
    0x00012c1040000480, 0x0000880802200804, 0x4000101190040800, 0x0214014208004084,
    0x0288400811422600, 0x9804100040400200, 0x020c410405042401, 0x0001094401030180,
    0x0010482424214000, 0x0c01008a90080010, 0x0001020084040000, 0x00000223a0880900,
    0x8042000c10442084, 0x0000049408261040, 0x0208885800b40020, 0x0950050a00820000,
    0x1001008814020210, 0x84000a6211042100, 0x0021408022081200, 0x40c0020000208800,
    0x400c0d0010203a00, 0x014201404a240100, 0x0100181810028200, 0x0444a00401020018,
];
#[cfg(not(feature = "pext"))]
#[rustfmt::skip]
const ROOK_MAGICS: [u64; Square::COUNT] = [
    0x0080001420400082, 0x1140016000401008, 0x0080100008802000, 0x2100048820100100,
    0x0200208410020028, 0x0200019004020048, 0x4100430012000094, 0x40800c2841800100,
    0x000c800080400c24, 0x4802002110448200, 0x2880801000200088, 0x0491002110004900,
    0x8020800802140180, 0x6000800200040081, 0x090a00080c032200, 0x4040800049000b80,
    0x2041248000400091, 0x0410004004200440, 0x9013010040200030, 0x8008008008801001,
    0x031080802c000800, 0x010280802200c400, 0x8070808011000200, 0x0200120005028044,
    0xc0004000800c2080, 0x0001d00340002000, 0x2002004200208010, 0x02080082801001b8,
    0x0001007100240800, 0x4000e20080800400, 0x1483004b00240200, 0x4c10800080026100,
    0x8085400820800080, 0x400860008a804000, 0x0410040800200020, 0x8100100059002100,
    0x0262013022000408, 0x0000402408011020, 0x0400881004008209, 0x084001108a001044,
    0x40828020c0008000, 0x0040100800206000, 0x3009208200420010, 0x400060d003010008,
    0x1000110008010005, 0x0002000418820010, 0x0000220004010100, 0x20000101884a0004,
    0x400200851021c200, 0x4009200180401c80, 0x0022324600208200, 0x1820082010050100,
    0x0904040080080080, 0x4210440080020080, 0x0688080b02300400, 0x8020309104c40200,
    0x0002910064800045, 0x504a410084620052, 0x040201208040100a, 0x8002002840102026,
    0x1001001210080005, 0x00c1006208440003, 0x0024111000883204, 0x0200050401618642,
];

#[derive(Clone, Copy)]
struct MagicEntry {
    pub mask: Bitboard,
    #[cfg(not(feature = "pext"))]
    pub magic: u64,
    #[cfg(not(feature = "pext"))]
    pub shift: u8,
    pub offset: usize,
}

const fn edge_bitboard(square: Square) -> Bitboard {
    const EDGE_FILES: Bitboard = File::A.to_bitboard().union(File::H.to_bitboard());
    const EDGE_RANKS: Bitboard = Rank::R1.to_bitboard().union(Rank::R8.to_bitboard());
    EDGE_FILES
        .diff(square.file().to_bitboard())
        .union(EDGE_RANKS.diff(square.rank().to_bitboard()))
}

const fn build_entries(kind: PieceKind) -> [MagicEntry; Square::COUNT] {
    let deltas = match kind {
        PieceKind::Bishop => &[-9, -7, 7, 9],
        PieceKind::Rook => &[-8, -1, 1, 8],
        _ => unreachable!(),
    };
    #[cfg(not(feature = "pext"))]
    let magics = match kind {
        PieceKind::Bishop => &BISHOP_MAGICS,
        PieceKind::Rook => &ROOK_MAGICS,
        _ => unreachable!(),
    };
    let mut entries = [MagicEntry {
        mask: Bitboard::EMPTY,
        #[cfg(not(feature = "pext"))]
        magic: 0,
        #[cfg(not(feature = "pext"))]
        shift: 0,
        offset: 0,
    }; 64];

    let mut curr_square = Some(Square::A1);
    let mut offset = 0;
    while let Some(square) = curr_square {
        let entry = &mut entries[square as usize];
        entry.mask = sliding_attacks(square, Bitboard::EMPTY, deltas).diff(edge_bitboard(square));
        #[cfg(not(feature = "pext"))]
        {
            entry.magic = magics[square as usize];
            entry.shift = 64 - entry.mask.popcount();
        }
        entry.offset = offset;
        offset += 1 << entry.mask.popcount();

        curr_square = square.shift(1);
    }

    entries
}

const BISHOP_SIZE: usize = 0x1480;
const BISHOP_ENTRIES: [MagicEntry; Square::COUNT] = build_entries(PieceKind::Bishop);

const ROOK_SIZE: usize = 0x19000;
const ROOK_ENTRIES: [MagicEntry; Square::COUNT] = build_entries(PieceKind::Rook);

#[inline]
#[cfg(not(feature = "pext"))]
const fn get_index(entry: &MagicEntry, occupied: Bitboard) -> usize {
    (entry
        .magic
        .wrapping_mul(occupied.intersect(entry.mask).value)
        >> entry.shift) as usize
        + entry.offset
}

#[cfg(feature = "pext")]
const fn get_index(entry: &MagicEntry, occupied: Bitboard) -> usize {
    occupied.value.extract_bits(entry.mask.value) as usize + entry.offset
}

const fn build_sliding_attacks<const N: usize>(
    entries: &[MagicEntry; 64],
    deltas: &[i8],
) -> [Bitboard; N] {
    let mut table = [Bitboard::EMPTY; N];

    let mut curr_square = Some(Square::A1);
    while let Some(square) = curr_square {
        let entry = &entries[square as usize];
        let mut occupied = Bitboard::EMPTY;
        loop {
            table[get_index(entry, occupied)] = sliding_attacks(square, occupied, deltas);

            if occupied.value == entry.mask.value {
                break;
            }
            occupied =
                Bitboard::new(occupied.value.wrapping_sub(entry.mask.value) & entry.mask.value);
        }

        curr_square = square.shift(1);
    }

    table
}

static BISHOP_ATTACKS: [Bitboard; BISHOP_SIZE] =
    build_sliding_attacks(&BISHOP_ENTRIES, &[-9, -7, 7, 9]);
static ROOK_ATTACKS: [Bitboard; ROOK_SIZE] = build_sliding_attacks(&ROOK_ENTRIES, &[-8, -1, 1, 8]);

/// Returns bishop attack squares considering blocking pieces using magic bitboards.
///
/// # Arguments
///
/// * `square` - The square the bishop is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the bishop can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Bitboard, attacks::bishop_attacks};
///
/// let attacks = bishop_attacks(Square::E4, Bitboard::EMPTY);
/// assert!(!attacks.is_empty());
/// ```
#[inline]
#[must_use]
pub const fn bishop_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let index = get_index(&BISHOP_ENTRIES[square as usize], occupied);
    debug_assert!(index < BISHOP_SIZE);
    unsafe {
        assert_unchecked(index < BISHOP_SIZE);
    }
    BISHOP_ATTACKS[index]
}

/// Returns rook attack squares considering blocking pieces using magic bitboards.
///
/// # Arguments
///
/// * `square` - The square the rook is on
/// * `occupied` - A bitboard of occupied squares
///
/// # Returns
///
/// Returns a bitboard of squares the rook can move to.
///
/// # Examples
///
/// ```
/// use smolchess::{Square, Bitboard, attacks::rook_attacks};
///
/// let attacks = rook_attacks(Square::E4, Bitboard::EMPTY);
/// assert!(!attacks.is_empty());
/// ```
#[inline]
#[must_use]
pub const fn rook_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    let index = get_index(&ROOK_ENTRIES[square as usize], occupied);
    debug_assert!(index < ROOK_SIZE);
    unsafe {
        assert_unchecked(index < ROOK_SIZE);
    }
    ROOK_ATTACKS[index]
}
