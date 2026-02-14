mod utils;


use std::fmt::Debug;
use std::collections::BTreeMap;
use std::fs::read_to_string;
use pipa::ir::{Op, gen_ir, dump_ir};
use pipa::error::{CompileError, ErrorReason};
use pipa::syntax::{Node, TokenType, ast};
use utils::{err_reason};


fn test_str(code: &str) -> Result<(), CompileError> {
    let nodes = ast(&code)?;

    gen_ir(&code, nodes)?;
    Ok(())
}


fn test_file(filename: &str) -> Result<(), CompileError> {
    let code = read_to_string(filename).unwrap();

    test_str(&code)?;
    Ok(())
}

// positive

#[test]
fn all_features() {
    assert_ok!(test_file("examples/all_features.pipa"));
}

#[test]
fn vars() {
    assert_ok!(test_file("examples/vars.pipa"));
}

#[test]
fn empty() {
    assert_ok!(test_file("examples/empty.pipa"));
}


#[test]
fn empty_without_newline() {
    assert_ok!(test_str(""));
}

#[test]
fn empty_code() {
    assert_ok!(test_file("examples/empty_code.pipa"));
}

#[test]
fn literal_after_empty_arr() {
    assert_ok!(test_file("examples/literal_after_empty_arr.pipa"));
}

#[test]
fn range_strings() {
    assert_ok!(test_file("examples/range_strings.pipa"));
}

#[test]
fn code_block_symbols_in_string() {
    assert_ok!(test_file("examples/code_block_symbols_in_string.pipa"));
}

#[test]
fn values_at_end_of_code_block() {
    assert_ok!(test_file("examples/values_at_end_of_code_block.pipa"));
}

#[test]
fn multiple_usage_of_macro() {
    assert_ok!(test_file("examples/multiple_usage_of_macro.pipa"));
}

#[test]
fn multiple_macro_definition() {
    assert_ok!(test_file("examples/multiple_macro_definition.pipa"));
}

#[test]
fn only_var() {
    assert_ok!(test_file("examples/only_var.pipa"));
}

#[test]
fn without_a_newline() {
    assert_ok!(test_str("{{ name }}"));
}

#[test]
fn utf_8() {
    assert_ok!(test_file("examples/utf-8.pipa"));
}

// negative

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

