use crate::{
    ast::Expr,
    effects::Effects,
    lexer, parser,
    typed_ast::TypedExpr,
    types::{Type, TypeEnv},
};

pub const SEMANTICS_VERSION: &str = "finlang-0.1";

#[derive(Debug, Clone, PartialEq)]
pub struct Compilation {
    pub source_version: &'static str,
    pub ast: Expr,
    pub typed_ast: TypedExpr,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub phase: &'static str,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(l), Some(c)) => write!(f, "{}:{}:{}: {}", self.phase, l, c, self.message),
            _ => write!(f, "{}: {}", self.phase, self.message),
        }
    }
}

pub fn compile_source(source: &str) -> Result<Compilation, Diagnostic> {
    let tokens = lexer::lex_spanned(source).map_err(|message| Diagnostic {
        phase: "lexer",
        message,
        line: None,
        column: None,
    })?;
    let plain: Vec<_> = tokens.iter().map(|t| t.token.clone()).collect();
    let ast = parser::parse(&plain).map_err(|message| Diagnostic {
        phase: "parser",
        message,
        line: None,
        column: None,
    })?;
    let mut env = TypeEnv::new();
    let mut effects = Effects::new();
    let (typed_ast, ty) = crate::type_checker::check_program_with_env(&ast, &mut env, &mut effects)
        .map_err(|message| Diagnostic {
            phase: "type-checker",
            message,
            line: None,
            column: None,
        })?;
    Ok(Compilation {
        source_version: SEMANTICS_VERSION,
        ast,
        typed_ast,
        ty,
    })
}

pub fn canonical_debug(c: &Compilation) -> String {
    format!(
        "version={} type={:?} ast={:?}",
        c.source_version, c.ty, c.ast
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline_compiles_mint() {
        let c = compile_source("mint(alice, 10)").unwrap();
        assert_eq!(c.source_version, SEMANTICS_VERSION);
    }
}
