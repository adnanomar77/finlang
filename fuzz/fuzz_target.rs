use std::io::{self, Read};
fn main() {
    let mut bytes=Vec::new(); io::stdin().read_to_end(&mut bytes).unwrap();
    let input=String::from_utf8_lossy(&bytes);
    let _=finlang_core::lexer::lex(&input);
    if let Ok(tokens)=finlang_core::lexer::lex(&input) {
        let _=finlang_core::parser::parse(&tokens);
        let _=finlang_core::compiler::compile_source(&input);
    }
}
