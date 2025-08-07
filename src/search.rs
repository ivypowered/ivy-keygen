use rand::{Rng, SeedableRng, rngs::StdRng};
use solana_pubkey::Pubkey;

const IVY_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("DkGdbW8SJmUoVE9KaBRwrvsQVhcuidy47DimjrhSoySE");
const GAME_PREFIX: &[u8] = b"game";
const GAME_MINT_PREFIX: &[u8] = b"game_mint";
const GAME_OTHER_PREFIXES: [&[u8]; 3] = [
    b"game_ivy_wallet",
    b"game_curve_wallet",
    b"game_treasury_wallet",
];
const TARGET_SUFFIX: &str = "ivy";

fn attempt(rng: &mut StdRng) -> Option<[u8; 32]> {
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

pub fn search() -> [u8; 32] {
    let mut rng = StdRng::from_os_rng();
    loop {
        match attempt(&mut rng) {
            Some(v) => return v,
            None => {}
        }
    }
}
