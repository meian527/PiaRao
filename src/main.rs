mod lexer;
mod ast;
mod parser;
mod interpreter;
mod lang;

fn main() {
    let src = String::from(
        r#"
        let x d = 0.1 + d;
        let result = x 0.2;
        let ok = result == 0.3;
        "#
    );
    let mut lang_state = lang::LangState::new(&src);
    lang_state.interpret();
    lang_state.print_var("ok", interpreter::ValueLiteral::Bool);
}
