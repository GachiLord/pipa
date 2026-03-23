use std::fs::{read_to_string};
use pipa::ir::gen_ir;
use pipa::syntax::{ast, TokenType};
use pipa::error::{CompileError, ErrorReason};
use pipa::analysis::{NO_OPT, FULL_OPT};
use pipa::utils::err_reason;

fn test_str(code: &str) -> Result<(), CompileError> {
    let nodes = ast(code)?;

    let c1 = gen_ir(code, nodes.clone(), NO_OPT);
    let c2 = gen_ir(code, nodes, FULL_OPT);

    assert_eq!(c1.is_err(), c2.is_err());

    match (&c1, &c2) {
        (Err(e1), Err(e2)) => {
            assert_eq!(e1, e2);
        },
        _ => {}
    }

    c1?;

    Ok(())
}

fn test_file(filename: &str) -> Result<(), CompileError> {
    let code = read_to_string(filename).unwrap();
    let r = test_str(&code);

    if let Err(ref e) = r {
        let mut output = Vec::new();

        e.write_message(&mut output, filename, &code).unwrap();
        println!("{}", String::from_utf8(output).unwrap());

        return r;
    }

    Ok(())
}

// names

#[test]
fn invalid_name() {
    assert_eq!(err_reason(test_file("negative_examples/invalid_name.pipa")), ErrorReason::NameError);
}

#[test]
fn array_names_in_strings() {
    assert_eq!(err_reason(test_file("negative_examples/array_names_in_strings.pipa")), ErrorReason::NameError);
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

// syntax

#[test]
fn missing_space_name_string() {
    assert_eq!(err_reason(test_file("negative_examples/missing_space_name_string.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Space, TokenType::NewLine, TokenType::RangeBegin] });
}

#[test]
fn broken_string() {
    assert_eq!(err_reason(test_file("negative_examples/broken_string.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Quote] }); 
}

#[test]
fn broken_string_piped() {
    assert_eq!(err_reason(test_file("negative_examples/broken_string_piped.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Quote] }); 
}

#[test]
fn array_no_braces() {
    assert_eq!(err_reason(test_file("negative_examples/array_no_braces.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::RangeBegin] }); 
}

#[test]
fn array_mismatched_brace() {
    assert_eq!(err_reason(test_file("negative_examples/array_mismatched_brace.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::Space, TokenType::NewLine, TokenType::RangeBegin] }); 
}

#[test]
fn unclosed_code_brace() {
    assert_eq!(err_reason(test_file("negative_examples/unclosed_code_brace.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::CodeEnd] }); 
}

#[test]
fn code_brace_mismatch() {
    assert_eq!(err_reason(test_file("negative_examples/code_brace_mismatch.pipa")), ErrorReason::SyntaxError { expected: vec![TokenType::CodeBegin] }); 
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
