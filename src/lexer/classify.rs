use crate::lexer::scanner::Lexemes;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tokens<'src> {
    Other(&'src str),
    Integer(i64),
    GreaterThen,
    SmalledThen,
    Ampersant,
    VerticalBar,
    FowardSlash,
    FatArrow,
    SemiColon,
    Colon,
    Comma,
    Star,
    Percent,
    Equals,
    Caret,
    Tilde,
    Dot,
    Mut,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Let,
    If,
    Else,
    Minus,
    Arrow,
    Fn,
    Less,
    Bigger,
    Struct,
    Enum,
    Tick,
    Plus,
    Type,
    Match,
    Const,
    EqualEqual,
    GreaterEqual,
    LessEqual,
    Or,
    And,
    ExclamationMark,
    As,
    QuestionMark,
}
#[derive(Clone, Debug)]
pub(crate) enum Integers {
    I32,
    I8,
    I16,
    I64,
    I128,
    Usize,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
}

pub(crate) fn classify_scanner<'a>(scanned: &[Lexemes<'a>]) -> Vec<Tokens<'a>> {
    let mut classified_lexemes = Vec::with_capacity(scanned.len());
    for item in scanned {
        let classified = match item {
            Lexemes::Other(value) => match *value {
                ">" => Tokens::GreaterThen,
                "<" => Tokens::SmalledThen,
                "->" => Tokens::Arrow,
                "fn" => Tokens::Fn,
                "let" => Tokens::Let,
                "as" => Tokens::As,
                "match" => Tokens::Match,
                "=>" => Tokens::FatArrow,
                _ => match value.parse::<i64>() {
                    Ok(integer) => Tokens::Integer(integer),
                    Err(_) => Tokens::Other(value),
                },
            },
            Lexemes::QuestionMark => Tokens::QuestionMark,
            Lexemes::Ampersant => Tokens::Ampersant,
            Lexemes::VerticalBar => Tokens::VerticalBar,
            Lexemes::FowardSlash => Tokens::FowardSlash,
            Lexemes::Exclamation => Tokens::ExclamationMark,
            Lexemes::SemiColon => Tokens::SemiColon,
            Lexemes::Colon => Tokens::Colon,
            Lexemes::Comma => Tokens::Comma,
            Lexemes::Tick => Tokens::Tick,
            Lexemes::Plus => Tokens::Plus,
            Lexemes::Minus => Tokens::Minus,
            Lexemes::Star => Tokens::Star,
            Lexemes::Percent => Tokens::Percent,
            Lexemes::Equals => Tokens::Equals,
            Lexemes::EqualEqual => Tokens::EqualEqual,
            Lexemes::GreaterEqual => Tokens::GreaterEqual,
            Lexemes::Less => Tokens::Less,
            Lexemes::Bigger => Tokens::Bigger,
            Lexemes::LessEqual => Tokens::LessEqual,
            Lexemes::Or => Tokens::Or,
            Lexemes::And => Tokens::And,
            Lexemes::Caret => Tokens::Caret,
            Lexemes::Tilde => Tokens::Tilde,
            Lexemes::Dot => Tokens::Dot,
            Lexemes::LeftParen => Tokens::LeftParen,
            Lexemes::RightParen => Tokens::RightParen,
            Lexemes::LeftBrace => Tokens::LeftBrace,
            Lexemes::RightBrace => Tokens::RightBrace,
            Lexemes::LeftBracket => Tokens::LeftBracket,
            Lexemes::RightBracket => Tokens::RightBracket,
            Lexemes::Struct => Tokens::Struct,
            Lexemes::Enum => Tokens::Enum,
            Lexemes::Type => Tokens::Type,
            Lexemes::Match => Tokens::Match,
            Lexemes::Mut => Tokens::Mut,
            Lexemes::Const => Tokens::Const,
            Lexemes::If => Tokens::If,
            Lexemes::Else => Tokens::Else,
        };
        classified_lexemes.push(classified);
    }

    classified_lexemes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_keywords_operators_and_integers() {
        let scanned = [
            Lexemes::Other("fn"),
            Lexemes::Other("let"),
            Lexemes::Other("as"),
            Lexemes::Other("=>"),
            Lexemes::Other("->"),
            Lexemes::Other("-42"),
            Lexemes::Other("name"),
            Lexemes::Less,
            Lexemes::Bigger,
        ];

        assert_eq!(
            classify_scanner(&scanned),
            vec![
                Tokens::Fn,
                Tokens::Let,
                Tokens::As,
                Tokens::FatArrow,
                Tokens::Arrow,
                Tokens::Integer(-42),
                Tokens::Other("name"),
                Tokens::Less,
                Tokens::Bigger,
            ]
        );
    }

    #[test]
    fn preserves_out_of_range_integer_as_other_text() {
        let scanned = [Lexemes::Other("999999999999999999999999999999")];
        assert_eq!(
            classify_scanner(&scanned),
            vec![Tokens::Other("999999999999999999999999999999")]
        );
    }
}
