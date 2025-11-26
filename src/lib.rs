mod syntax;
mod ir;
mod vm;
mod error;

use std::collections::BTreeMap;
use std::fs::read_to_string;
pub use ir::{gen_ir, dump_ir};
pub use error::CompileError;
pub use syntax::ast;
pub use vm::{Vm, VmError};

fn test_file(filename: &str) {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();
    let code = read_to_string(filename).unwrap();

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
        Ok(_) => {},
        Err(e) => {
            dump_ir(&ir);
            vm.dump_state();
            dbg!(e);
            assert_eq!(e, VmError::EndOfProgram);
        }
    }
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
