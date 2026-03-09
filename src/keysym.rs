/// Resolve an XKB keysym name to its Unicode character.
///
/// Handles the following formats, in order:
///
/// 1. **Named keysyms** — looked up in the `x11-keysymdef` database
///    (e.g. `"ampersand"` → `'&'`, `"eacute"` → `'é'`).
/// 2. **Single printable ASCII characters** — returned as-is
///    (e.g. `"a"` → `'a'`).
/// 3. **`U<hex>` Unicode notation** — 1–6 hex digits after a leading `U`
///    (e.g. `"U458"` → `'ј'`, `"U0041"` → `'A'`, `"U1F600"` → `'😀'`).
/// 4. **`0x<hex>` XKB keysym notation** — if the value is `≥ 0x1000000`,
///    the Unicode code point is `value - 0x1000000`; otherwise the value
///    is used directly as the code point
///    (e.g. `"0x10FFFFB"` → `U+FFFFB`, `"0x0041"` → `'A'`).
///
/// Returns `None` if the name cannot be resolved by any of the above methods.
pub fn keysym_name_to_char(name: &str) -> Option<char> {
    // 1. Named keysym lookup via x11-keysymdef.
    if let Some(record) = x11_keysymdef::lookup_by_name(name) {
        if record.unicode != '\0' {
            return Some(record.unicode);
        }
    }

    // 2. Single printable ASCII character.
    if name.len() == 1 {
        return name.chars().next();
    }

    // 3. U<hex> Unicode notation: "U" followed by 1–6 hex digits.
    if let Some(hex) = name.strip_prefix('U') {
        if !hex.is_empty() && hex.len() <= 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
        }
    }

    // 4. 0x<hex> XKB keysym notation.
    //    Keysym values >= 0x1000000 encode Unicode as (value - 0x1000000).
    //    Values below that threshold are used directly as the code point.
    if let Some(hex) = name.strip_prefix("0x").or_else(|| name.strip_prefix("0X")) {
        if !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(value) = u32::from_str_radix(hex, 16) {
                let codepoint = if value >= 0x1000000 { value - 0x1000000 } else { value };
                return char::from_u32(codepoint);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_keysym() {
        assert_eq!(keysym_name_to_char("ampersand"), Some('&'));
        assert_eq!(keysym_name_to_char("eacute"), Some('é'));
    }

    #[test]
    fn single_char() {
        assert_eq!(keysym_name_to_char("a"), Some('a'));
        assert_eq!(keysym_name_to_char("Z"), Some('Z'));
    }

    #[test]
    fn unicode_notation_four_digits() {
        assert_eq!(keysym_name_to_char("U0041"), Some('A'));
        assert_eq!(keysym_name_to_char("U1F600"), Some('😀'));
    }

    #[test]
    fn unicode_notation_three_digits() {
        // U458 = CYRILLIC SMALL LETTER JE (ј)
        assert_eq!(keysym_name_to_char("U458"), Some('ј'));
        // U408 = CYRILLIC CAPITAL LETTER JE (Ј)
        assert_eq!(keysym_name_to_char("U408"), Some('Ј'));
    }

    #[test]
    fn hex_notation_short() {
        assert_eq!(keysym_name_to_char("0x0041"), Some('A'));
        assert_eq!(keysym_name_to_char("0x00e9"), Some('é'));
    }

    #[test]
    fn hex_notation_xkb_encoded() {
        // 0x10FFFFB = XKB keysym for U+FFFFB (value - 0x1000000 = 0xFFFFB)
        assert_eq!(keysym_name_to_char("0x10FFFFB"), char::from_u32(0xFFFFB));
        // 0x10FFFFD = XKB keysym for U+FFFFD
        assert_eq!(keysym_name_to_char("0x10FFFFD"), char::from_u32(0xFFFfd));
        // 0x1000967 = XKB keysym for U+0967 (Nepali digit one, १)
        assert_eq!(keysym_name_to_char("0x1000967"), Some('१'));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(keysym_name_to_char("NoSymbol"), None);
        assert_eq!(keysym_name_to_char("VoidSymbol"), None);
    }
}
