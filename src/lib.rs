pub mod syntax;
pub mod ir;
pub mod vm;
pub mod error;

use std::collections::BTreeMap;
use std::fs::read_to_string;
use ir::{gen_ir, dump_ir};
use syntax::ast;
use vm::{Vm, VmError};

fn test_str(filename: &str, code: &str) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();

    // tokenize + lex
    let tokens = match ast(&code) {
        Ok(r) => r, 
        Err(e) => { 
            e.write_message(&mut handle, filename, &code).unwrap();
            panic!();
        }
    };

    // ir
    let ir = match gen_ir(&code, tokens) {
        Ok(ir) => ir,
        Err(e) => { 
            e.write_message(&mut handle, filename, &code).unwrap();
            panic!();
        }
    };

    // run
    let vars = BTreeMap::from([
        ("name".into(), "cake".into()),
        ("sirname".into(), "cakolas".into()),
        ("arg".into(), "this is an arg".into()),
    ]);
    let arrays = BTreeMap::from([
        ("ARGS".into(), vec!["cake".into(), "cakolas".into(), "this is an arg".into()]),
    ]);
    let mut vm = Vm::new(vars, arrays);

    match vm.run(&mut handle, &ir) {
        Ok(_) => {
        },
        Err(e) => {
            dump_ir(&mut handle, &ir).unwrap();
            vm.dump_state(&mut handle).unwrap();
            dbg!(e);
            assert_eq!(e, VmError::EndOfProgram);
        }
    }
}

fn test_file(filename: &str) {
    let code = read_to_string(filename).unwrap();

    test_str(filename, &code);
}

#[test]
fn all_features() {
    test_file("examples/all_features.pipa");
}

#[test]
fn vars() {
    test_file("examples/vars.pipa");
}

#[test]
fn empty() {
    test_file("examples/empty.pipa");
}

#[test]
fn empty_code() {
    test_file("examples/empty_code.pipa");
}

#[test]
fn literal_after_empty_arr() {
    test_file("examples/literal_after_empty_arr.pipa");
}

#[test]
fn range_strings() {
    test_file("examples/range_strings.pipa");
}

#[test]
#[should_panic]
fn undefined_scope() {
    test_file("examples/undefined_scope.pipa");
}

#[test]
#[should_panic]
fn undefined_scope_with_range() {
    test_file("examples/undefined_scope_with_range.pipa");
}

#[test]
#[should_panic]
fn broken_string() {
    test_file("examples/broken_string.pipa");
}


#[test]
fn code_block_symbols_in_string() {
    test_file("examples/code_block_symbols_in_string.pipa");
}

#[test]
fn values_at_end_of_code_block() {
    test_file("examples/values_at_end_of_code_block.pipa");
}

#[test]
fn multiple_usage_of_macro() {
    test_file("examples/multiple_usage_of_macro.pipa");
}

#[test]
fn multiple_macro_definition() {
    test_file("examples/multiple_macro_definition.pipa");
}

#[test]
fn only_var() {
    test_file("examples/only_var.pipa");
}

#[test]
fn without_a_newline() {
    test_str("examples/without_a_newline.pipa", "{{ name }}");
}

#[test]
fn utf_8() {
    test_file("examples/utf-8.pipa");
}
