#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let,
    In,
    Fn,
    Policy,
    If,
    Then,
    Else,
    True,
    False,
    Transfer,
    Mint,
    OracleRead,
    Validate,
    ToAmount,
    UnsafeAssumeTrusted,
    CreateLoan,
    Repay,
    PriceUpdate,
    Liquidate,
    LParen,
    RParen,
    Comma,
    Equals,
    EqEq,
    Lt,
    Gt,
    Le,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Colon,
    Semicolon,
    Arrow,
    LBrace,
    RBrace,
    Number(u64),
    Float(f64),
    Ident(String),
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

fn keyword(s: &str) -> Option<Token> {
    Some(match s {
        "let" => Token::Let,
        "in" => Token::In,
        "fn" => Token::Fn,
        "policy" => Token::Policy,
        "if" => Token::If,
        "then" => Token::Then,
        "else" => Token::Else,
        "true" => Token::True,
        "false" => Token::False,
        "transfer" => Token::Transfer,
        "mint" => Token::Mint,
        "oracleRead" => Token::OracleRead,
        "validate" => Token::Validate,
        "toAmount" => Token::ToAmount,
        "unsafeAssumeTrusted" => Token::UnsafeAssumeTrusted,
        "createLoan" => Token::CreateLoan,
        "repay" => Token::Repay,
        "priceUpdate" => Token::PriceUpdate,
        "liquidate" => Token::Liquidate,
        _ => return None,
    })
}

pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    Ok(lex_spanned(input)?.into_iter().map(|t| t.token).collect())
}

pub fn lex_spanned(input: &str) -> Result<Vec<SpannedToken>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;
    while i < chars.len() {
        let start = Span {
            line,
            column: col,
            offset: i,
        };
        let c = chars[i];
        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            col += 1;
            continue;
        }
        if c == '\n' {
            i += 1;
            line += 1;
            col = 1;
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                col += 1;
            }
            continue;
        }
        let tok = match c {
            '(' => {
                i += 1;
                col += 1;
                Token::LParen
            }
            ')' => {
                i += 1;
                col += 1;
                Token::RParen
            }
            ',' => {
                i += 1;
                col += 1;
                Token::Comma
            }
            '=' => {
                i += 1;
                col += 1;
                if i < chars.len() && chars[i] == '=' {
                    i += 1;
                    col += 1;
                    Token::EqEq
                } else {
                    Token::Equals
                }
            }
            '<' => {
                i += 1;
                col += 1;
                if i < chars.len() && chars[i] == '=' {
                    i += 1;
                    col += 1;
                    Token::Le
                } else {
                    Token::Lt
                }
            }
            '>' => {
                i += 1;
                col += 1;
                if i < chars.len() && chars[i] == '=' {
                    i += 1;
                    col += 1;
                    Token::Ge
                } else {
                    Token::Gt
                }
            }
            '+' => {
                i += 1;
                col += 1;
                Token::Plus
            }
            '-' => {
                i += 1;
                col += 1;
                if i < chars.len() && chars[i] == '>' {
                    i += 1;
                    col += 1;
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            '*' => {
                i += 1;
                col += 1;
                Token::Star
            }
            '/' => {
                i += 1;
                col += 1;
                Token::Slash
            }
            '{' => {
                i += 1;
                col += 1;
                Token::LBrace
            }
            '}' => {
                i += 1;
                col += 1;
                Token::RBrace
            }
            ':' => {
                i += 1;
                col += 1;
                Token::Colon
            }
            ';' => {
                i += 1;
                col += 1;
                Token::Semicolon
            }
            '0'..='9' => {
                let mut s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    s.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                if i < chars.len() && chars[i] == '.' {
                    s.push('.');
                    i += 1;
                    col += 1;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        s.push(chars[i]);
                        i += 1;
                        col += 1;
                    }
                    Token::Float(s.parse().map_err(|_| {
                        format!("Invalid decimal at {}:{}", start.line, start.column)
                    })?)
                } else {
                    Token::Number(s.parse().map_err(|_| {
                        format!("Invalid number at {}:{}", start.line, start.column)
                    })?)
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut s = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    s.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                keyword(&s).unwrap_or(Token::Ident(s))
            }
            _ => return Err(format!("Unexpected character '{}' at {}:{}", c, line, col)),
        };
        out.push(SpannedToken {
            token: tok,
            span: start,
        });
    }
    out.push(SpannedToken {
        token: Token::EOF,
        span: Span {
            line,
            column: col,
            offset: i,
        },
    });
    Ok(out)
}

pub use crate::ast::{AssetKind, Currency, PolicyId, SourceId};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexes_all_operations() {
        let t = lex("createLoan(a,b,l,1,asset,200,1.5); repay(a,b,l,1);").unwrap();
        assert!(t.contains(&Token::CreateLoan));
        assert!(t.contains(&Token::Float(1.5)));
    }
    #[test]
    fn reports_location() {
        assert!(lex("mint(a, @)").unwrap_err().contains("1:9"));
    }
}
