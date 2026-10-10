#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lexemes<'a> {
    Other(&'a str),
    Ampersant,   //&
    VerticalBar, // |
    FowardSlash, // /
    Exclamation,
    SemiColon,
    Colon,
    Comma,
    Plus,
    Minus,
    Star, // *
    Percent,
    Equals,
    Caret, // ^
    Tilde,
    Dot,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    Tick,
    RightBracket,
    Struct,
    Less,
    Bigger,
    Enum,
    Type,
    Match,
    Mut,
    Const,
    If,
    Else,
    EqualEqual,
    GreaterEqual,
    LessEqual,
    Or,
    QuestionMark,
    And,
}

pub(crate) fn create_lexemes<'src>(text: &'src str) -> Vec<Lexemes<'src>> {
    let count = text.split_whitespace().count();
    let mut lexemes_collection = Vec::with_capacity(count);

    for word in text.split_whitespace() {
        let lexeme = match word {
            "&" => Lexemes::Ampersant,
            "|" => Lexemes::VerticalBar,
            "/" => Lexemes::FowardSlash,
            "!" => Lexemes::Exclamation,
            ";" => Lexemes::SemiColon,
            ":" => Lexemes::Colon,
            "?" => Lexemes::QuestionMark,
            "," => Lexemes::Comma,
            "+" => Lexemes::Plus,
            "-" => Lexemes::Minus,
            "*" => Lexemes::Star,
            "%" => Lexemes::Percent,
            "=" => Lexemes::Equals,
            "==" => Lexemes::EqualEqual,
            ">=" => Lexemes::GreaterEqual,
            "<=" => Lexemes::LessEqual,
            "<" => Lexemes::Less,
            ">" => Lexemes::Bigger,
            "||" => Lexemes::Or,
            "&&" => Lexemes::And,
            "^" => Lexemes::Caret,
            "~" => Lexemes::Tilde,
            "." => Lexemes::Dot,
            "(" => Lexemes::LeftParen,
            ")" => Lexemes::RightParen,
            "{" => Lexemes::LeftBrace,
            "}" => Lexemes::RightBrace,
            "[" => Lexemes::LeftBracket,
            "]" => Lexemes::RightBracket,
            "struct" => Lexemes::Struct,
            "enum" => Lexemes::Enum,
            "type" => Lexemes::Type,
            "match" => Lexemes::Match,
            "mut" => Lexemes::Mut,
            "const" => Lexemes::Const,
            "if" => Lexemes::If,
            "else" => Lexemes::Else,
            "'" => Lexemes::Tick,
            other => Lexemes::Other(other),
        };
        lexemes_collection.push(lexeme);
    }

    lexemes_collection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_keywords_symbols_and_other_text() {
        assert_eq!(
            create_lexemes("fn name -> i64 { let x = 1 ; }"),
            vec![
                Lexemes::Other("fn"),
                Lexemes::Other("name"),
                Lexemes::Other("->"),
                Lexemes::Other("i64"),
                Lexemes::LeftBrace,
                Lexemes::Other("let"),
                Lexemes::Other("x"),
                Lexemes::Equals,
                Lexemes::Other("1"),
                Lexemes::SemiColon,
                Lexemes::RightBrace,
            ]
        );
    }

    #[test]
    fn scans_all_recognized_multi_character_symbols() {
        assert_eq!(
            create_lexemes("== >= <= || &&"),
            vec![
                Lexemes::EqualEqual,
                Lexemes::GreaterEqual,
                Lexemes::LessEqual,
                Lexemes::Or,
                Lexemes::And,
            ]
        );
    }
}
