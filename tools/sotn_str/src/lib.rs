mod generated_codec;

use generated_codec::{
    DAKUTEN_COMPOSITIONS, DAKUTEN_MARK, ESCAPE_BYTE, HANDAKUTEN_COMPOSITIONS, HANDAKUTEN_MARK,
    LITERAL_ESCAPE_GLYPH, PSP_FONT_TABLE, PSX_FONT_TABLE,
};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Platform {
    Psx,
    Psp,
}

impl Platform {
    pub fn parse(value: &str) -> Result<Self, CodecError> {
        match value {
            "psx" => Ok(Self::Psx),
            "psp" => Ok(Self::Psp),
            _ => Err(CodecError::new(format!(
                "unsupported string platform {value:?}; expected psx or psp"
            ))),
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Psx => f.write_str("psx"),
            Self::Psp => f.write_str("psp"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodecError(String);

impl CodecError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for CodecError {}

fn platform_data(platform: Platform) -> &'static [char; 256] {
    match platform {
        Platform::Psx => &PSX_FONT_TABLE,
        Platform::Psp => &PSP_FONT_TABLE,
    }
}

fn remove_voice_mark(composed: char, pairs: &[(char, char)]) -> Option<char> {
    pairs
        .iter()
        .find_map(|(base, candidate)| (*candidate == composed).then_some(*base))
}

fn compose_voice_mark(base: char, mark: u8) -> Option<char> {
    let pairs = match mark {
        DAKUTEN_MARK => DAKUTEN_COMPOSITIONS,
        HANDAKUTEN_MARK => HANDAKUTEN_COMPOSITIONS,
        _ => return None,
    };
    pairs
        .iter()
        .find_map(|(candidate, composed)| (*candidate == base).then_some(*composed))
}

fn index_for(table: &[char; 256], value: char) -> Option<u8> {
    // Prefer the last table occurrence, matching the historical sotn_str lookup.
    table
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, candidate)| (*candidate == value).then_some(index as u8))
}

/// Encode a string for the game's 8x8 menu font.
///
/// The returned bytes include the game-string terminator (`0xFF`) but not the
/// C string's trailing NUL.
pub fn encode_menu_string(value: &str, platform: Platform) -> Result<Vec<u8>, CodecError> {
    let table = platform_data(platform);
    let mut encoded = Vec::with_capacity(value.len() + 1);

    for character in value.chars() {
        if character == LITERAL_ESCAPE_GLYPH {
            encoded.extend([ESCAPE_BYTE, ESCAPE_BYTE]);
            continue;
        }
        let (base, mark) = if let Some(base) = remove_voice_mark(character, DAKUTEN_COMPOSITIONS) {
            (Some(base), DAKUTEN_MARK)
        } else if let Some(base) = remove_voice_mark(character, HANDAKUTEN_COMPOSITIONS) {
            (Some(base), HANDAKUTEN_MARK)
        } else {
            (None, 0)
        };
        if let Some(base) = base {
            let index = index_for(table, base).ok_or_else(|| {
                CodecError::new(format!(
                    "character {character:?} requires unavailable base {base:?} on {platform}"
                ))
            })?;
            encoded.extend([index, ESCAPE_BYTE, mark]);
            continue;
        }
        let index = index_for(table, character).ok_or_else(|| {
            CodecError::new(format!(
                "character {character:?} is not available on {platform}"
            ))
        })?;
        encoded.push(index);
    }
    encoded.push(ESCAPE_BYTE);
    Ok(encoded)
}

/// Decode one terminated 8x8-menu-font string.
///
/// This accepts either the raw game terminator (`0xFF` at end of input) or the
/// in-memory C representation (`0xFF 0x00`). Trailing data is rejected.
pub fn decode_menu_string(value: &[u8], platform: Platform) -> Result<String, CodecError> {
    let table = platform_data(platform);
    let mut decoded = Vec::new();
    let mut offset = 0;

    while offset < value.len() {
        let byte = value[offset];
        offset += 1;
        if byte != ESCAPE_BYTE {
            decoded.push(table[byte as usize]);
            continue;
        }
        if offset == value.len() {
            return Ok(decoded.into_iter().collect());
        }
        let escaped = value[offset];
        offset += 1;
        match escaped {
            0 => {
                if offset != value.len() {
                    return Err(CodecError::new("trailing bytes after string terminator"));
                }
                return Ok(decoded.into_iter().collect());
            }
            ESCAPE_BYTE => decoded.push(LITERAL_ESCAPE_GLYPH),
            DAKUTEN_MARK | HANDAKUTEN_MARK => {
                let base = decoded.last_mut().ok_or_else(|| {
                    CodecError::new(format!(
                        "voice mark 0x{escaped:02X} has no preceding character"
                    ))
                })?;
                *base = compose_voice_mark(*base, escaped).ok_or_else(|| {
                    CodecError::new(format!(
                        "cannot apply voice mark 0x{escaped:02X} to {base:?}"
                    ))
                })?;
            }
            _ => {
                return Err(CodecError::new(format!(
                    "unknown string escape 0xFF 0x{escaped:02X}"
                )))
            }
        }
    }
    Err(CodecError::new("unterminated string"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Vectors {
        encode: Vec<EncodeVector>,
        decode: Vec<DecodeVector>,
        invalid_encode: Vec<InvalidEncodeVector>,
        invalid_decode: Vec<InvalidDecodeVector>,
    }

    #[derive(Deserialize)]
    struct EncodeVector {
        name: String,
        text: String,
        psx: Option<String>,
        psp: Option<String>,
    }

    #[derive(Deserialize)]
    struct DecodeVector {
        name: String,
        bytes: String,
        psx: Option<String>,
        psp: Option<String>,
    }

    #[derive(Deserialize)]
    struct InvalidEncodeVector {
        name: String,
        text: String,
        platforms: Vec<String>,
    }

    #[derive(Deserialize)]
    struct InvalidDecodeVector {
        name: String,
        bytes: String,
        platforms: Vec<String>,
    }

    fn bytes(value: &str) -> Vec<u8> {
        assert_eq!(value.len() % 2, 0);
        (0..value.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
            .collect()
    }

    fn vectors() -> Vectors {
        serde_json::from_str(include_str!("../../sotn_codec/conformance.json")).unwrap()
    }

    fn for_platform<'a, T>(
        psx: &'a Option<T>,
        psp: &'a Option<T>,
    ) -> [(Platform, &'a Option<T>); 2] {
        [(Platform::Psx, psx), (Platform::Psp, psp)]
    }

    #[test]
    fn shared_encode_vectors() {
        for vector in vectors().encode {
            for (platform, expected) in for_platform(&vector.psx, &vector.psp) {
                if let Some(expected) = expected {
                    assert_eq!(
                        encode_menu_string(&vector.text, platform).unwrap(),
                        bytes(expected),
                        "{} ({platform})",
                        vector.name
                    );
                }
            }
        }
    }

    #[test]
    fn shared_decode_vectors() {
        for vector in vectors().decode {
            for (platform, expected) in for_platform(&vector.psx, &vector.psp) {
                if let Some(expected) = expected {
                    assert_eq!(
                        decode_menu_string(&bytes(&vector.bytes), platform).unwrap(),
                        *expected,
                        "{} ({platform})",
                        vector.name
                    );
                }
            }
        }
    }

    #[test]
    fn shared_invalid_vectors() {
        let vectors = vectors();
        for vector in vectors.invalid_encode {
            for platform in vector.platforms {
                assert!(
                    encode_menu_string(&vector.text, Platform::parse(&platform).unwrap()).is_err(),
                    "{} ({platform})",
                    vector.name
                );
            }
        }
        for vector in vectors.invalid_decode {
            for platform in vector.platforms {
                assert!(
                    decode_menu_string(&bytes(&vector.bytes), Platform::parse(&platform).unwrap())
                        .is_err(),
                    "{} ({platform})",
                    vector.name
                );
            }
        }
    }
}
