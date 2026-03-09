use crate::{ast::*, xkb::Rule};
use derivative::Derivative;
use pest_ast::FromPest;

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::definition))]
pub struct Definition<'src> {
    pub modifiers: BlockModifiers<'src>,
    pub directive: Directive<'src>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::block_modifiers))]
pub struct BlockModifiers<'src> {
    pub values: Vec<BlockModifier<'src>>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::block_modifier))]
pub struct BlockModifier<'src> {
    #[pest_ast(outer(with(span_into_str)))]
    pub content: &'src str,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::directive))]
pub enum Directive<'src> {
    #[derivative(Debug = "transparent")]
    XkbSymbols(XkbSymbols<'src>),
    #[derivative(Debug = "transparent")]
    XkbKeycodes(XkbKeycodes<'src>),
    #[derivative(Debug = "transparent")]
    XkbTypes(XkbTypes<'src>),
    #[derivative(Debug = "transparent")]
    XkbCompatibility(XkbCompatibility<'src>),
    #[derivative(Debug = "transparent")]
    XkbGeometry(XkbGeometry<'src>),
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::include))]
pub struct Include<'src> {
    pub name: StringContent<'src>,
}

impl<'src> Include<'src> {
    /// Parse the include string into a file/locale name and an optional layout
    /// variant.
    ///
    /// XKB include strings follow the pattern `locale` or `locale(variant)`,
    /// e.g. `"us"` → `("us", None)` and `"us(basic)"` → `("us", Some("basic"))`.
    pub fn parse_name(&self) -> (&str, Option<&str>) {
        parse_include(self.name.content)
    }
}

/// Split an XKB include string into a file/locale name and an optional layout
/// variant.
///
/// XKB include strings follow the pattern `locale` or `locale(variant)`,
/// e.g. `"us"` → `("us", None)` and `"us(basic)"` → `("us", Some("basic"))`.
pub fn parse_include(input: &str) -> (&str, Option<&str>) {
    match input.find('(') {
        None => (input, None),
        Some(paren_open) => {
            let locale = &input[..paren_open];
            let rest = &input[paren_open + 1..];
            let variant = rest.trim_end_matches(')');
            (locale, Some(variant))
        }
    }
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::override_))]
pub struct Override<'src> {
    pub name: StringContent<'src>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::augment))]
pub struct Augment<'src> {
    pub name: StringContent<'src>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::virtual_modifiers))]
pub struct VirtualModifiers<'src> {
    pub name: Vec<KeyCombo<'src>>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::action))]
pub struct Action<'src> {
    pub name: Ident<'src>,
    pub params: Vec<ActionParam<'src>>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::action_param))]
pub enum ActionParam<'src> {
    #[derivative(Debug = "transparent")]
    ParamAssignment(ParamAssignment<'src>),
    #[derivative(Debug = "transparent")]
    ParamExpression(ParamExpression<'src>),
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::param_assignment))]
pub struct ParamAssignment<'src> {
    pub ident: Ident<'src>,
    pub expr: ParamExpression<'src>,
}

#[derive(Derivative, FromPest, Clone, PartialEq)]
#[derivative(Debug)]
#[pest_ast(rule(Rule::param_expression))]
pub struct ParamExpression<'src> {
    #[pest_ast(inner(with(span_into_str)))]
    pub content: &'src str,
}
