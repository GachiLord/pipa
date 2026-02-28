mod utils;

use std::fs::{read_to_string};
use pipa::ir::gen_ir;
use pipa::syntax::{ast, TokenType};
use pipa::error::{CompileError, ErrorReason};
use utils::err_reason;

fn test_str(code: &str) -> Result<(), CompileError> {
    let nodes = ast(code)?;

    let _ = gen_ir(code, nodes)?;
    Ok(())
}

fn test_file(filename: &str) -> Result<(), CompileError> {
    let code = read_to_string(filename).unwrap();

    test_str(&code)?;
    Ok(())
}


#[test]
fn undefined_scope() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope.pipa")), ErrorReason::UndefinedVar { name: "_name".into() });
}

#[test]
fn undefined_scope_single() {

    assert_eq!(err_reason(test_file("negative_examples/undefined_scope_single.pipa")), ErrorReason::UndefinedVar { name: "_".into() });
}

#[test]
fn undefined_scope_with_range() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope_with_range.pipa")), ErrorReason::UndefinedVar { name: "_name".into() });
}

#[test]
fn broken_string() {
    assert_eq!(err_reason(test_file("negative_examples/broken_string.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Quote] }); 
}

#[test]
fn nested_macro() {
    assert_eq!(err_reason(test_file("negative_examples/nested_macro.pipa")), ErrorReason::NestedMacro); 
}

#[test]
fn empty_macro() {
    assert_eq!(err_reason(test_file("negative_examples/empty_macro.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Space] }); 
}
