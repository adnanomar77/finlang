use finlang_core::{
    bytecode::{compile, verify, Vm},
    compiler::compile_source,
    lexer::lex,
    state::FinancialState,
};

#[test]
fn generated_numeric_programs_preserve_compilation_and_execution() {
    let mut seed = 0x9e3779b97f4a7c15u64;
    for _ in 0..500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let a = seed % 10_000;
        seed = seed.rotate_left(17);
        let b = seed % 10_000;
        let source = format!("{} + {}", a, b);
        let c = compile_source(&source).unwrap();
        let code = compile(&c.typed_ast);
        assert!(verify(&code).is_ok());
        let mut vm = Vm::new();
        let mut s = FinancialState::new();
        let _ = vm.execute(&code, &mut s).unwrap();
    }
}

#[test]
fn lexer_never_panics_on_generated_ascii() {
    let alphabet = b"abcXYZ0123_(),=+-*/<> \n";
    for n in 0..1000 {
        let text: String = (0..(n % 64))
            .map(|i| alphabet[(i * 31 + n) % alphabet.len()] as char)
            .collect();
        let _ = lex(&text);
    }
}
