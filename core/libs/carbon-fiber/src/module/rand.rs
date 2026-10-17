use rand_chacha::ChaCha8Rng;
use rand_core::{
  Rng,
  SeedableRng,
};
use sha2::{
  Digest,
  Sha256,
};

/// Uses the provided MCU Serial and Ocilation drift to create a cryptograpically secure pairing pin and salt for PAKE handshakes (respectively).
pub fn raw_generate_pairing_code_and_salt(
  unique_chip_id: &[u8],
  trng_noise: &[u8],
) -> (u32, [u8; 16]) {
  let mut hasher = Sha256::new();

  hasher.update(unique_chip_id);
  hasher.update(trng_noise);

  let seed: [u8; 32] = hasher.finalize().into();
  let mut rng = ChaCha8Rng::from_seed(seed);

  let pin = 100_000 + (rng.next_u32() % 900_000);
  let mut salt = [0u8; 16];

  rng.fill_bytes(&mut salt);

  (pin, salt)
}
