use anyhow::Context;
use rand::seq::{IndexedRandom, SliceRandom};

const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijkmnopqrstuvwxyz";
const NUMBER: &[u8] = b"123456789";
const SYMBOL: &[u8] = b"!@#$%^&*_";

/// # Errors
#[expect(clippy::fn_params_excessive_bools)]
pub fn process_genpass(
    length: u8,
    no_upper: bool,
    no_lower: bool,
    no_number: bool,
    no_symbol: bool,
) -> anyhow::Result<String> {
    let mut rng = rand::rng();
    let mut password = Vec::new();
    let mut chars = Vec::new();

    if !no_upper {
        chars.extend_from_slice(UPPER);
        password.push(*UPPER.choose(&mut rng).context("UPPER won't be empty")?);
    }

    if !no_lower {
        chars.extend_from_slice(LOWER);
        password.push(*LOWER.choose(&mut rng).context("LOWER won't be empty")?);
    }
    if !no_number {
        chars.extend_from_slice(NUMBER);
        password.push(*NUMBER.choose(&mut rng).context("NUMBER won't be empty")?);
    }
    if !no_symbol {
        chars.extend_from_slice(SYMBOL);
        password.push(*SYMBOL.choose(&mut rng).context("SYMBOL won't be empty")?);
    }

    if chars.is_empty() {
        anyhow::bail!("At least one character set must be enabled");
    }

    let seeded = password.len();
    let length = length as usize;
    if length < seeded {
        anyhow::bail!("length {length} is too short: {seeded} character set(s) enabled");
    }
    for _ in 0..(length - seeded) {
        let c = chars
            .choose(&mut rng)
            .context("chars won't be empty in this context")?;
        password.push(*c);
    }

    password.shuffle(&mut rng);

    let password = String::from_utf8(password)?;

    Ok(password)
}
