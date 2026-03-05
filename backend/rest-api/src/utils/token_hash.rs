use sha2::{Digest, Sha256};

/// Returns the SHA-256 hex digest of a raw refresh token.
pub fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    hex::encode(digest)
}

/// Minimal hex encoder (avoids pulling the `hex` crate for one function).
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .fold(String::with_capacity(64), |mut acc, b| {
                use std::fmt::Write;
                let _ = write!(acc, "{b:02x}");
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_64_char_hex_string() {
        let hash = sha256_hex("some-token");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn deterministic_output() {
        assert_eq!(sha256_hex("hello"), sha256_hex("hello"));
    }

    #[test]
    fn different_inputs_different_outputs() {
        assert_ne!(sha256_hex("token-a"), sha256_hex("token-b"));
    }

    #[test]
    fn empty_input_produces_valid_hash() {
        let hash = sha256_hex("");
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
