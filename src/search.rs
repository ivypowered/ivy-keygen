use rand::{Rng, SeedableRng, rngs::StdRng};
use solana_pubkey::Pubkey;

const IVY_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("DkGdbW8SJmUoVE9KaBRwrvsQVhcuidy47DimjrhSoySE");

// Game prefixes
const GAME_PREFIX: &[u8] = b"game";
const GAME_MINT_PREFIX: &[u8] = b"game_mint";
const GAME_OTHER_PREFIXES: [&[u8]; 3] = [
    b"game_ivy_wallet",
    b"game_curve_wallet",
    b"game_treasury_wallet",
];

// Sync prefixes
const SYNC_PREFIX: &[u8] = b"sync";
const SYNC_MINT_PREFIX: &[u8] = b"sync_mint";
const SYNC_OTHER_PREFIXES: [&[u8]; 2] = [b"sync_sync_wallet", b"sync_pump_wallet"];

const TARGET_SUFFIX: &str = "ivy";

fn attempt_game(rng: &mut StdRng) -> Option<[u8; 32]> {
    let seed: [u8; 32] = rng.random();
    let game = Pubkey::create_program_address(&[GAME_PREFIX, &seed], &IVY_PROGRAM_ID)
        .ok()?
        .to_bytes();
    let mint = Pubkey::create_program_address(&[GAME_MINT_PREFIX, &game], &IVY_PROGRAM_ID).ok()?;
    if !mint.to_string().ends_with(TARGET_SUFFIX) {
        return None;
    }
    for game_prefix in GAME_OTHER_PREFIXES {
        Pubkey::create_program_address(&[game_prefix, &game], &IVY_PROGRAM_ID).ok()?;
    }
    Some(seed)
}

fn attempt_sync(rng: &mut StdRng) -> Option<[u8; 32]> {
    let seed: [u8; 32] = rng.random();
    let sync = Pubkey::create_program_address(&[SYNC_PREFIX, &seed], &IVY_PROGRAM_ID)
        .ok()?
        .to_bytes();
    let mint = Pubkey::create_program_address(&[SYNC_MINT_PREFIX, &sync], &IVY_PROGRAM_ID).ok()?;
    if !mint.to_string().ends_with(TARGET_SUFFIX) {
        return None;
    }
    for sync_prefix in SYNC_OTHER_PREFIXES {
        Pubkey::create_program_address(&[sync_prefix, &sync], &IVY_PROGRAM_ID).ok()?;
    }
    Some(seed)
}

pub fn search_game() -> [u8; 32] {
    let mut rng = StdRng::from_os_rng();
    loop {
        if let Some(seed) = attempt_game(&mut rng) {
            return seed;
        }
    }
}

pub fn search_sync() -> [u8; 32] {
    let mut rng = StdRng::from_os_rng();
    loop {
        if let Some(seed) = attempt_sync(&mut rng) {
            return seed;
        }
    }
}
