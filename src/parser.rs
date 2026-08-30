use crate::{ast::*, lexer::Token};
pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    let mut p = Parser { tokens, pos: 0 };
    let e = p.expr()?;
    if matches!(p.peek(), Token::Semicolon) {
        p.pos += 1;
    }
    if !matches!(p.peek(), Token::EOF) {
        return Err(p.err("Unexpected tokens after expression"));
    }
    Ok(e)
}
struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}
impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::EOF)
    }
    fn take(&mut self) -> Token {
        let t = self.peek().clone();
        self.pos += 1;
        t
    }
    fn err(&self, s: &str) -> String {
        format!("{} at token {}", s, self.pos + 1)
    }
    fn expect(&mut self, f: fn(&Token) -> bool, s: &str) -> Result<(), String> {
        if f(self.peek()) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(s))
        }
    }
    fn ident(&mut self, s: &str) -> Result<String, String> {
        match self.take() {
            Token::Ident(x) => Ok(x),
            _ => Err(self.err(s)),
        }
    }
    fn expr(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Token::Policy) {
            return self.policy_definition();
        }
        if matches!(self.peek(), Token::Let) {
            self.pos += 1;
            let n = self.ident("Expected name")?;
            self.expect(|t| matches!(t, Token::Equals), "Expected '='")?;
            let v = self.expr()?;
            self.expect(|t| matches!(t, Token::In), "Expected 'in'")?;
            let b = self.expr()?;
            return Ok(Expr::Let {
                name: n,
                value: Box::new(v),
                body: Box::new(b),
            });
        }
        if matches!(self.peek(), Token::If) {
            self.pos += 1;
            let c = self.expr()?;
            self.expect(|t| matches!(t, Token::Then), "Expected 'then'")?;
            let t = self.expr()?;
            self.expect(|t| matches!(t, Token::Else), "Expected 'else'")?;
            let e = self.expr()?;
            return Ok(Expr::If {
                condition: Box::new(c),
                then_branch: Box::new(t),
                else_branch: Box::new(e),
            });
        }
        self.compare()
    }
    fn compare(&mut self) -> Result<Expr, String> {
        let mut e = self.add()?;
        loop {
            let op = match self.peek() {
                Token::Lt => BinOp::Lt,
                Token::Gt => BinOp::Gt,
                Token::Le => BinOp::Le,
                Token::Ge => BinOp::Ge,
                Token::EqEq => BinOp::Eq,
                _ => break,
            };
            self.pos += 1;
            let r = self.add()?;
            e = Expr::Binary {
                op,
                left: Box::new(e),
                right: Box::new(r),
            };
        }
        Ok(e)
    }
    fn add(&mut self) -> Result<Expr, String> {
        let mut e = self.mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.pos += 1;
            let r = self.mul()?;
            e = Expr::Binary {
                op,
                left: Box::new(e),
                right: Box::new(r),
            };
        }
        Ok(e)
    }
    fn mul(&mut self) -> Result<Expr, String> {
        let mut e = self.atom()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.pos += 1;
            let r = self.atom()?;
            e = Expr::Binary {
                op,
                left: Box::new(e),
                right: Box::new(r),
            };
        }
        Ok(e)
    }
    fn atom(&mut self) -> Result<Expr, String> {
        match self.take() {
            Token::Number(n) => Ok(Expr::Int(n)),
            Token::True => Ok(Expr::Bool(true)),
            Token::False => Ok(Expr::Bool(false)),
            Token::Fn => self.function_literal(),
            Token::Ident(n) => {
                if matches!(self.peek(), Token::LParen) {
                    self.lparen()?;
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            args.push(self.expr()?);
                            if matches!(self.peek(), Token::Comma) {
                                self.pos += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    self.rparen()?;
                    Ok(Expr::Call {
                        function: Box::new(Expr::Var { name: n }),
                        args,
                    })
                } else {
                    Ok(Expr::Var { name: n })
                }
            }
            Token::LParen => {
                let e = self.expr()?;
                self.expect(|t| matches!(t, Token::RParen), "Expected ')'")?;
                Ok(e)
            }
            Token::Mint => self.call2(|a, x| Expr::Mint {
                account: a,
                amount: Box::new(x),
            }),
            Token::Transfer => self.call3(|a, b, x| Expr::Transfer {
                from: a,
                to: b,
                asset: Box::new(x),
            }),
            Token::OracleRead => {
                self.lparen()?;
                let f = self.source()?;
                self.rparen()?;
                Ok(Expr::OracleRead { feed: f })
            }
            Token::Validate => {
                self.lparen()?;
                let o = self.expr()?;
                self.comma()?;
                let p = self.policy()?;
                self.rparen()?;
                Ok(Expr::Validate {
                    oracle: Box::new(o),
                    policy: p,
                })
            }
            Token::ToAmount => {
                self.lparen()?;
                let x = self.expr()?;
                self.rparen()?;
                Ok(Expr::ToAmount {
                    verified: Box::new(x),
                })
            }
            Token::UnsafeAssumeTrusted => {
                self.lparen()?;
                let x = self.expr()?;
                self.rparen()?;
                Ok(Expr::UnsafeAssumeTrusted {
                    oracle: Box::new(x),
                })
            }
            Token::CreateLoan => {
                self.pos -= 1;
                self.loan()
            }
            Token::Repay => {
                self.pos -= 1;
                self.repay()
            }
            Token::PriceUpdate => {
                self.pos -= 1;
                self.price()
            }
            Token::Liquidate => {
                self.pos -= 1;
                self.liquidate()
            }
            t => Err(self.err(&format!("Unexpected token {:?}", t))),
        }
    }
    fn policy_definition(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        let name = self.ident("Expected policy name")?;
        self.lparen()?;
        let parameter = self.ident("Expected policy parameter")?;
        if matches!(self.peek(), Token::Colon) {
            self.pos += 1;
            let _ = self.ident("Expected policy parameter type")?;
        }
        self.rparen()?;
        self.expect(|t| matches!(t, Token::LBrace), "Expected '{'")?;
        let predicate = self.expr()?;
        self.expect(|t| matches!(t, Token::RBrace), "Expected '}'")?;
        self.expect(|t| matches!(t, Token::In), "Expected 'in' after policy")?;
        let body = self.expr()?;
        Ok(Expr::PolicyDef {
            name,
            parameter,
            predicate: Box::new(predicate),
            body: Box::new(body),
        })
    }
    fn function_literal(&mut self) -> Result<Expr, String> {
        self.lparen()?;
        let mut params = Vec::new();
        if !matches!(self.peek(), Token::RParen) {
            loop {
                let p = self.ident("Expected parameter name")?;
                if matches!(self.peek(), Token::Colon) {
                    self.pos += 1;
                    let _ = self.ident("Expected parameter type")?;
                }
                params.push(p);
                if matches!(self.peek(), Token::Comma) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        self.rparen()?;
        self.expect(|t| matches!(t, Token::Arrow), "Expected '->'")?;
        let _return_type = self.ident("Expected return type")?;
        self.expect(|t| matches!(t, Token::LBrace), "Expected '{'")?;
        let body = self.expr()?;
        self.expect(|t| matches!(t, Token::RBrace), "Expected '}'")?;
        Ok(Expr::Function {
            params,
            body: Box::new(body),
        })
    }
    fn lparen(&mut self) -> Result<(), String> {
        self.expect(|t| matches!(t, Token::LParen), "Expected '('")
    }
    fn rparen(&mut self) -> Result<(), String> {
        self.expect(|t| matches!(t, Token::RParen), "Expected ')'")
    }
    fn comma(&mut self) -> Result<(), String> {
        self.expect(|t| matches!(t, Token::Comma), "Expected ','")
    }
    fn call2<F: FnOnce(String, Expr) -> Expr>(&mut self, f: F) -> Result<Expr, String> {
        self.lparen()?;
        let a = self.ident("Expected identifier")?;
        self.comma()?;
        let x = self.expr()?;
        self.rparen()?;
        Ok(f(a, x))
    }
    fn call3<F: FnOnce(String, String, Expr) -> Expr>(&mut self, f: F) -> Result<Expr, String> {
        self.lparen()?;
        let a = self.ident("Expected identifier")?;
        self.comma()?;
        let b = self.ident("Expected identifier")?;
        self.comma()?;
        let x = self.expr()?;
        self.rparen()?;
        Ok(f(a, b, x))
    }
    fn source(&mut self) -> Result<SourceId, String> {
        match self.ident("Expected feed")?.as_str() {
            "feedA" => Ok(SourceId::FeedA),
            "feedB" => Ok(SourceId::FeedB),
            _ => Err(self.err("Expected feedA or feedB")),
        }
    }
    fn policy(&mut self) -> Result<PolicyId, String> {
        match self.ident("Expected policy")?.as_str() {
            "PriceBounds" => Ok(PolicyId::PriceBounds),
            x => Ok(PolicyId::Named(x.to_string())),
        }
    }
    fn loan(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        self.lparen()?;
        let a = self.ident("Expected borrower")?;
        self.comma()?;
        let p = self.ident("Expected pool")?;
        self.comma()?;
        let id = self.ident("Expected loan id")?;
        self.comma()?;
        let am = self.expr()?;
        self.comma()?;
        let ca = self.ident("Expected collateral")?;
        self.comma()?;
        let cv = self.expr()?;
        self.comma()?;
        let rr = match self.take() {
            Token::Number(n) => n as f64,
            Token::Float(x) => x,
            _ => return Err(self.err("Expected ratio")),
        };
        self.rparen()?;
        Ok(Expr::CreateLoan {
            borrower: a,
            lender_pool: p,
            loan_id: id,
            amount: Box::new(am),
            collateral_asset: ca,
            collateral_value: Box::new(cv),
            required_ratio: rr,
        })
    }
    fn repay(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        self.lparen()?;
        let a = self.ident("Expected borrower")?;
        self.comma()?;
        let p = self.ident("Expected pool")?;
        self.comma()?;
        let l = self.expr()?;
        self.comma()?;
        let x = self.expr()?;
        self.rparen()?;
        Ok(Expr::Repay {
            borrower: a,
            lender_pool: p,
            loan: Box::new(l),
            payment: Box::new(x),
        })
    }
    fn price(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        self.lparen()?;
        let l = self.expr()?;
        self.comma()?;
        let x = self.expr()?;
        self.rparen()?;
        Ok(Expr::PriceUpdate {
            loan: Box::new(l),
            new_price: Box::new(x),
        })
    }
    fn liquidate(&mut self) -> Result<Expr, String> {
        self.pos += 1;
        self.lparen()?;
        let l = self.expr()?;
        self.rparen()?;
        Ok(Expr::Liquidate { loan: Box::new(l) })
    }
}
