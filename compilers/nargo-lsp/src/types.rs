//! Nargo LSP types.

use oak_core::TokenType;

/// Token types for Nargo language lexical analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NargoToken {
    /// Whitespace characters.
    Whitespace,
    /// Comments.
    Comment,
    /// Keywords.
    Keyword,
    /// Identifiers.
    Identifier,
    /// String literals.
    String,
    /// Numeric literals.
    Number,
    /// Operators.
    Operator,
    /// Punctuation.
    Punctuation,
    /// End of stream.
    EndOfStream,
    /// Unknown token.
    Unknown,
}

/// Element types for Nargo language structural analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NargoElement {
    /// Root element.
    Root,
    /// Template section.
    Template,
    /// Script section.
    Script,
    /// Style section.
    Style,
    /// Tag element.
    Tag,
    /// Attribute.
    Attribute,
    /// Expression.
    Expression,
    /// Token wrapper.
    Token(NargoToken),
    /// Error element.
    Error,
}

impl From<NargoToken> for NargoElement {
    fn from(t: NargoToken) -> Self {
        Self::Token(t)
    }
}

/// Nargo language definition.
pub struct NargoLanguage;

impl NargoLanguage {
    /// Language name.
    pub const NAME: &'static str = "nargo";
}

impl oak_core::Language for NargoLanguage {
    const NAME: &'static str = "nargo";
    const CATEGORY: oak_core::language::LanguageCategory = oak_core::language::LanguageCategory::Markup;

    type TokenType = NargoToken;
    type ElementType = NargoElement;
    type TypedRoot = ();
}

impl oak_core::language::TokenType for NargoToken {
    type Role = oak_core::language::UniversalTokenRole;

    const END_OF_STREAM: Self = Self::EndOfStream;

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace => oak_core::language::UniversalTokenRole::Whitespace,
            Self::Comment => oak_core::language::UniversalTokenRole::Comment,
            Self::Keyword => oak_core::language::UniversalTokenRole::Keyword,
            Self::Identifier => oak_core::language::UniversalTokenRole::Name,
            Self::String => oak_core::language::UniversalTokenRole::Literal,
            Self::Number => oak_core::language::UniversalTokenRole::Literal,
            Self::Operator => oak_core::language::UniversalTokenRole::Operator,
            Self::Punctuation => oak_core::language::UniversalTokenRole::Punctuation,
            Self::EndOfStream => oak_core::language::UniversalTokenRole::Eof,
            Self::Unknown => oak_core::language::UniversalTokenRole::Error,
        }
    }
}

impl oak_core::language::ElementType for NargoElement {
    type Role = oak_core::language::UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => oak_core::language::UniversalElementRole::Root,
            Self::Template => oak_core::language::UniversalElementRole::Container,
            Self::Script => oak_core::language::UniversalElementRole::Container,
            Self::Style => oak_core::language::UniversalElementRole::Container,
            Self::Tag => oak_core::language::UniversalElementRole::Container,
            Self::Attribute => oak_core::language::UniversalElementRole::Attribute,
            Self::Expression => oak_core::language::UniversalElementRole::Expression,
            Self::Token(token) => oak_core::language::UniversalElementRole::None,
            Self::Error => oak_core::language::UniversalElementRole::Error,
        }
    }
}
