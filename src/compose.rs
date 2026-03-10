use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

/// A single parsed Compose entry.
///
/// - `keys` is the resolved key sequence (each element is the `char` that
///   the corresponding keysym token resolves to).
/// - `multi_key_index` is the index within `keys` at which `Multi_key`
///   appeared, if present. `None` means no `Multi_key` was in the sequence.
///   `Some(0)` is the common leading compose-key case. `Some(n)` for n > 0
///   indicates a mid-sequence compose key (preserved for callers).
/// - `output` is the character this sequence should emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeEntry {
    pub keys: Vec<char>,
    pub multi_key_index: Option<usize>,
    pub output: char,
}

/// Parse an X11-style Compose file at `path` and return the list of
/// resolved `ComposeEntry`s.
///
/// Behavior:
/// - Blank lines and lines starting with `#` are ignored.
/// - `include "..."` directives are followed recursively. Relative include
///   paths are resolved relative to the containing file's directory.
/// - Each rule line of the form `<keys...> : "output" ...` or
///   `<keys...> : KEYSYM_NAME` is parsed. Key tokens like `<A>` are resolved
///   via `keysym_name_to_char`. `Multi_key` tokens are recorded via
///   `multi_key_index` and are not inserted into `keys`.
/// - Entries with unresolvable keysyms or empty `keys` are skipped. If the
///   output cannot be resolved to a `char` the entry is skipped.
///
/// This function never panics on I/O errors; it treats unreadable files as
/// absent and simply returns whatever entries were accumulated.
pub fn parse_compose_file(path: &Path) -> Vec<ComposeEntry> {
    let mut out = Vec::new();
    parse_compose_file_impl(path, &mut out);
    out
}

fn parse_compose_file_impl(path: &Path, out: &mut Vec<ComposeEntry>) {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle include "..." directives
        if trimmed.starts_with("include") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end_rel) = trimmed[start + 1..].find('"') {
                    let include_path_str = &trimmed[start + 1..start + 1 + end_rel];
                    let include_path = Path::new(include_path_str);
                    let resolved = if include_path.is_absolute() {
                        include_path.to_path_buf()
                    } else if let Some(parent) = path.parent() {
                        parent.join(include_path)
                    } else {
                        include_path.to_path_buf()
                    };
                    parse_compose_file_impl(&resolved, out);
                }
            }
            continue;
        }

        // Only consider lines that start with '<' (key sequence)
        if !trimmed.starts_with('<') {
            continue;
        }

        // Split "<key...> ... : value"
        let (keys_part, value_part) = match trimmed.split_once(':') {
            Some(pair) => pair,
            None => continue,
        };

        let keys_str = keys_part.trim();
        let value_str = value_part.trim();

        let mut keys: Vec<char> = Vec::new();
        let mut multi_key_index: Option<usize> = None;
        let mut skip_entry = false;

        for token in keys_str.split_whitespace() {
            let name = token.trim_start_matches('<').trim_end_matches('>');
            if name == "Multi_key" {
                // Record where in the keys vector the Multi_key appeared.
                // The index is the number of keys accumulated so far.
                if multi_key_index.is_none() {
                    multi_key_index = Some(keys.len());
                } else {
                    // Multiple Multi_key tokens — preserve the first index and continue.
                }
                continue;
            }
            // Resolve keysym name to char using the crate-provided helper.
            // Use the public helper exported from this crate.
            match crate::keysym_name_to_char(name) {
                Some(c) => keys.push(c),
                None => {
                    // Unresolvable keysym — skip this entry entirely.
                    skip_entry = true;
                    break;
                }
            }
        }

        if skip_entry || keys.is_empty() {
            continue;
        }

        // Resolve the output character.
        if let Some(output_char) = parse_compose_output(value_str) {
            out.push(ComposeEntry { keys, multi_key_index, output: output_char });
        }
    }
}

/// Parse the right-hand side of a compose rule and return the resulting
/// `char` if it can be resolved.
///
/// Supported forms:
/// - Quoted string: "\"x\"" -> 'x' (with extra heuristics matching the
///   previous implementation: if the quoted string is non-empty and doesn't
///   start with a backslash and doesn't start with a digit, the first
///   character is returned).
/// - Keysym name (unquoted): `quoted_name` or `U2039` etc. Delegates to
///   `keysym_name_to_char`.
fn parse_compose_output(value_str: &str) -> Option<char> {
    // If it doesn't start with a quote, treat it as a keysym name directly.
    if !value_str.starts_with('"') {
        let name = value_str.split_whitespace().next().unwrap_or("").trim();
        if name.is_empty() || name.starts_with('#') {
            return None;
        }
        return crate::keysym_name_to_char(name);
    }

    // value_str starts with a quote: find the closing quote.
    let rest = &value_str[1..];
    let end_quote = match rest.find('"') {
        Some(i) => i,
        None => return None,
    };
    let inner = &rest[..end_quote];

    // If the quoted string is simple (non-empty, doesn't start with backslash,
    // and the first char is not an ASCII digit), return its first char.
    if !inner.is_empty() && !inner.starts_with('\\') {
        if let Some(first) = inner.chars().next() {
            if !first.is_ascii_digit() {
                return Some(first);
            }
        }
    }

    // Otherwise, after the quoted string there may be a keysym name to use.
    let after_quote = rest[end_quote + 1..].trim();
    if after_quote.is_empty() || after_quote.starts_with('#') {
        return None;
    }
    let keysym_name = after_quote.split_whitespace().next()?;
    if keysym_name.starts_with('#') {
        return None;
    }
    crate::keysym_name_to_char(keysym_name)
}
