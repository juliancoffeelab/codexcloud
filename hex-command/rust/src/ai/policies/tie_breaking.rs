//! Deterministic tie-breaking policy shared deliberately by tactical and recruitment evaluators.
pub struct Random(pub u32);
impl Random {
    pub fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b79f5);
        let mut v = (self.0 ^ (self.0 >> 15)).wrapping_mul(1 | self.0);
        v ^= v.wrapping_add((v ^ (v >> 7)).wrapping_mul(61 | v));
        (v ^ (v >> 14)) as f64 / 4294967296.0
    }
}
pub(in crate::ai) fn jitter(seed: u32, key: &str) -> f64 {
    let mut h = seed;
    for c in key.encode_utf16() {
        h = (h ^ c as u32).wrapping_mul(16777619);
    }
    h as f64 / 4294967296.0 * 0.05
}
fn hash_number(mut hash: u32, mut number: u32) -> u32 {
    let mut digits = [0; 10];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (number % 10) as u8;
        number /= 10;
        if number == 0 {
            break;
        }
    }
    for &byte in &digits[start..] {
        hash = (hash ^ byte as u32).wrapping_mul(16777619);
    }
    hash
}
/// Hash geometric action facts, independently of authored tile identity.
pub(in crate::ai) fn action_jitter(
    seed: u32,
    round: u32,
    unit: u32,
    tile: (i32, i32),
    target: Option<u32>,
) -> f64 {
    let byte = |hash: u32, byte: u8| (hash ^ byte as u32).wrapping_mul(16777619);
    let mut hash = hash_number(seed, round);
    hash = byte(byte(hash, b':'), b'u');
    hash = hash_number(hash, unit);
    hash = byte(hash, b':');
    for (index, coordinate) in [tile.0, tile.1].into_iter().enumerate() {
        if index != 0 {
            hash = byte(hash, b',');
        }
        if coordinate < 0 {
            hash = byte(hash, b'-');
        }
        hash = hash_number(hash, coordinate.unsigned_abs());
    }
    hash = byte(hash, b':');
    if let Some(target) = target {
        hash = hash_number(byte(hash, b'u'), target);
    } else {
        for character in b"move" {
            hash = byte(hash, *character);
        }
    }
    hash as f64 / 4294967296.0 * 0.05
}

#[cfg(test)]
mod tests {
    use super::{Random, action_jitter, jitter};
    #[test]
    fn action_jitter_matches_formatted_position_keys() {
        for seed in [0, 19, u32::MAX] {
            for (round, unit) in [(0, 0), (30, 12), (u32::MAX, u32::MAX)] {
                for tile in [(3, 2), (0, 0), (-10, 100)] {
                    for target in [None, Some(0), Some(u32::MAX)] {
                        let key = format!(
                            "{round}:u{unit}:{},{}:{}",
                            tile.0,
                            tile.1,
                            target.map_or("move".into(), |id| format!("u{id}"))
                        );
                        assert_eq!(
                            action_jitter(seed, round, unit, tile, target),
                            jitter(seed, &key)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn seeded_random_is_reproducible_and_bounded() {
        for seed in [0, 1, u32::MAX] {
            let mut left = Random(seed);
            let mut right = Random(seed);
            for _ in 0..32 {
                let value = left.next();
                assert_eq!(value, right.next());
                assert!((0.0..1.0).contains(&value));
            }
        }
    }
}
