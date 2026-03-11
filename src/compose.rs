use std::{fs, path::Path};

use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

/// A compact Compose entry type used by callers/tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeEntry {
    pub keys: Vec<char>,
    pub keysym_names: Vec<String>,
    pub multi_key_index: Option<usize>,
    pub output: char,
}

#[derive(Parser)]
#[grammar = "src/compose.pest"]
struct ComposeParser;

/// Parse a compose file and return resolved entries.
pub fn parse_compose_file(path: &Path) -> Vec<ComposeEntry> {
    let mut out = Vec::new();
    parse_compose_file_impl(path, &mut out);
    out
}

fn parse_compose_file_impl(path: &Path, out: &mut Vec<ComposeEntry>) {
    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return,
    };

    let pairs = match ComposeParser::parse(Rule::file, &content) {
        Ok(p) => p,
        Err(_) => return,
    };

    for top in pairs {
        for line in top.into_inner() {
            match line.as_rule() {
                Rule::include_line => {
                    if let Some(s) = line.into_inner().find(|p| p.as_rule() == Rule::string) {
                        let include_str = strip_quotes(s.as_str());
                        if include_str.is_empty() {
                            continue;
                        }
                        let include_path = Path::new(include_str);
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
                Rule::rule_line => {
                    let mut inner = line.into_inner();
                    let keys_pair = inner.find(|p| p.as_rule() == Rule::keys);
                    let value_pair = inner.find(|p| p.as_rule() == Rule::value);
                    if let (Some(kp), Some(vp)) = (keys_pair, value_pair) {
                        if let Some((keys, names, multi_index)) = resolve_keys(kp) {
                            let (opt_out, _name) = resolve_value(vp);
                            if let Some(ch) = opt_out {
                                out.push(ComposeEntry {
                                    keys,
                                    keysym_names: names,
                                    multi_key_index: multi_index,
                                    output: ch,
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Resolve keys: returns (chars, keysym names, optional multi_key index)
fn resolve_keys(pair: Pair<Rule>) -> Option<(Vec<char>, Vec<String>, Option<usize>)> {
    let mut chars = Vec::new();
    let mut names = Vec::new();
    let mut multi_idx: Option<usize> = None;

    for key_token in pair.into_inner().filter(|p| p.as_rule() == Rule::key_token) {
        // key_token -> key_ident
        let ident = key_token.into_inner().find(|p| p.as_rule() == Rule::key_ident)?;
        let name = ident.as_str().trim();
        if name.eq_ignore_ascii_case("Multi_key") {
            if multi_idx.is_none() {
                multi_idx = Some(chars.len());
            }
            continue;
        }
        if let Some(c) = crate::keysym_name_to_char(name) {
            chars.push(c);
            names.push(name.to_string());
        } else {
            return None;
        }
    }

    if chars.is_empty() {
        None
    } else {
        Some((chars, names, multi_idx))
    }
}

/// Resolve RHS value: return (char, backing keysym name)
fn resolve_value(pair: Pair<Rule>) -> (Option<char>, String) {
    let mut parts = pair.into_inner();
    let first = match parts.next() {
        Some(p) => p,
        None => return (None, String::new()),
    };

    match first.as_rule() {
        Rule::string => {
            let s = strip_quotes(first.as_str());
            if !s.is_empty() && !s.starts_with('\\') {
                if let Some(ch) = s.chars().next() {
                    if !ch.is_ascii_digit() {
                        return (Some(ch), s.to_string());
                    }
                }
            }
            if let Some(next) = parts.next() {
                if next.as_rule() == Rule::keysym_name {
                    let name = next.as_str();
                    return (crate::keysym_name_to_char(name), name.to_string());
                }
            }
            (None, s.to_string())
        }
        Rule::keysym_name => {
            let name = first.as_str();
            (crate::keysym_name_to_char(name), name.to_string())
        }
        _ => (None, first.as_str().to_string()),
    }
}

#[inline]
fn strip_quotes(s: &str) -> &str {
    s.strip_prefix('"').and_then(|t| t.strip_suffix('"')).unwrap_or(s)
}
