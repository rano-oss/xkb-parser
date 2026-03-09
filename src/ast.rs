use crate::xkb::Rule;
use derivative::Derivative;
use pest_ast::FromPest;

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::file))]
pub struct File<'src> {
    pub definitions: Vec<Definition<'src>>,
    #[derivative(Debug = "ignore")]
    eoi: EOI,
}

impl<'src> File<'src> {
    /// Returns the name of every `xkb_symbols` block in this file, in
    /// document order, without any filtering.
    pub fn symbol_layout_names(&self) -> Vec<&str> {
        self.definitions
            .iter()
            .filter_map(|d| {
                if let Directive::XkbSymbols(sym) = &d.directive {
                    Some(sym.name.content)
                } else {
                    None
                }
            })
            .collect()
    }
}

mod helpers;
pub(crate) use helpers::*;

mod basic;
pub use basic::*;

mod common;
pub use common::*;

mod xkb_symbols;
pub use xkb_symbols::*;

mod xkb_keycodes;
pub use xkb_keycodes::*;

mod xkb_types;
pub use xkb_types::*;

mod xkb_compatibility;
pub use xkb_compatibility::*;

mod xkb_geometry;
pub use xkb_geometry::*;

#[cfg(test)]
mod tests;
