/// Resolve an XKB keysym name to its Unicode character.
///
/// Handles the following formats, in order:
///
/// 1. **Named keysyms** — looked up in the `x11-keysymdef` database
///    (e.g. `"ampersand"` → `'&'`, `"eacute"` → `'é'`).
/// 2. **Single printable ASCII characters** — returned as-is
///    (e.g. `"a"` → `'a'`).
/// 3. **`U<hex>` Unicode notation** — 4–6 hex digits after a leading `U`
///    (e.g. `"U0041"` → `'A'`, `"U1F600"` → `'😀'`).
/// 4. **`0x<hex>` hexadecimal notation** — last four hex digits used as
///    the code point (e.g. `"0x0041"` → `'A'`).
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

    // 3. U<hex> Unicode notation: "U" followed by 4–6 hex digits.
    if let Some(hex) = name.strip_prefix('U') {
        if hex.len() >= 4 && hex.len() <= 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return u32::from_str_radix(hex, 16).ok().and_then(char::from_u32);
        }
    }

    // 4. 0x<hex> hexadecimal notation: use the last four hex digits.
    if let Some(hex) = name.strip_prefix("0x").or_else(|| name.strip_prefix("0X")) {
        if hex.len() >= 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let start = hex.len().saturating_sub(4);
            return u32::from_str_radix(&hex[start..], 16).ok().and_then(char::from_u32);
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
    fn unicode_notation() {
        assert_eq!(keysym_name_to_char("U0041"), Some('A'));
        assert_eq!(keysym_name_to_char("U1F600"), Some('😀'));
    }

    #[test]
    fn hex_notation() {
        assert_eq!(keysym_name_to_char("0x0041"), Some('A'));
        assert_eq!(keysym_name_to_char("0x00e9"), Some('é'));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(keysym_name_to_char("NoSymbol"), None);
        assert_eq!(keysym_name_to_char("VoidSymbol"), None);
    }
}
