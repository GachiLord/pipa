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

// pipes

#[test]
fn undefined_scope() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope.pipa")), ErrorReason::UndefinedVar { name: "_name".into() });
}

#[test]
fn undefined_scope_single() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope_single.pipa")), ErrorReason::UndefinedVar { name: "_".into() });
}

#[test]
fn undefined_scope_single_with_range() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope_single_with_range.pipa")), ErrorReason::UndefinedVar { name: "_".into() });
}

#[test]
fn undefined_scope_with_range() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_scope_with_range.pipa")), ErrorReason::UndefinedVar { name: "_name".into() });
}

#[test]
fn orphan_pipe() {
    assert_eq!(err_reason(test_file("negative_examples/orphan_pipe.pipa")), ErrorReason::PipeNoParent);
}

#[test]
fn array_not_piped() {
    assert_eq!(err_reason(test_file("negative_examples/array_not_piped.pipa")), ErrorReason::ArrayNotPiped);
}

#[test]
fn array_no_new_line() {
    assert_eq!(err_reason(test_file("negative_examples/array_no_new_line.pipa")), ErrorReason::ArrayNoNewLine);
}

// strings

#[test]
fn broken_string() {
    assert_eq!(err_reason(test_file("negative_examples/broken_string.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Quote] }); 
}

#[test]
fn broken_string_piped() {
    assert_eq!(err_reason(test_file("negative_examples/broken_string_piped.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Quote] }); 
}

// macro

#[test]
fn nested_macro() {
    assert_eq!(err_reason(test_file("negative_examples/nested_macro.pipa")), ErrorReason::NestedMacro); 
}

#[test]
fn empty_macro() {
    assert_eq!(err_reason(test_file("negative_examples/empty_macro.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Space] }); 
}


#[test]
fn undefined_macro() {
    assert_eq!(err_reason(test_file("negative_examples/undefined_macro.pipa")), ErrorReason::UndefinedMacro { name: "print".into() }); 
}


#[test]
fn macro_redifinition() {
    assert_eq!(err_reason(test_file("negative_examples/macro_redifinition.pipa")), ErrorReason::MacroRedefinition { name: "print".into() }); 
}
