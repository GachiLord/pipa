use std::io::{stdout, Write};
use std::fs::{read_to_string, read_dir};
use pipa::ir::gen_ir;
use pipa::syntax::ast;
use pipa::vm::Vm;
use pipa::analysis::{NO_OPT, FULL_OPT};
use pipa::utils::{VARS, ARRAYS};
use std::env;


fn test_str(f: &mut impl Write, filename: &str, code: &str, output: &str) {

    let nodes = match ast(&code) {
        Ok(n) => n,
        Err(e) => {
            e.write_message(f, filename, code).unwrap();
            panic!();
        }
    };

    let ir = match gen_ir(&code, nodes.clone(), NO_OPT) {
        Ok(ir) => ir,
        Err(e) => {
            e.write_message(f, filename, code).unwrap();
            panic!();
        },
    };

    if env::var("TEST_SKIP_SNAPSHOT").unwrap_or_default() != "" {
        return;
    }
    
    let mut out = Vec::new();
    let mut vm = Vm::new(VARS.clone(), ARRAYS.clone());

    match vm.run(&mut out, &ir) {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to run '{}' {:?}", filename, e);
        },
    }

    // check diff with no optimizations
    let out_s = String::from_utf8(out).expect(&format!("Output of '{}' is not utf8", filename));
    assert_eq!(out_s, output, "{}\n{:#?}\n{:#?}\n{:#?}", filename, &ir, nodes, NO_OPT);
    // check diff with full optimizations
    let ir_opt = match gen_ir(&code, nodes.clone(), FULL_OPT) {
        Ok(ir) => ir,
        Err(e) => {
            e.write_message(f, filename, code).unwrap();
            panic!();
        },
    };

    
    let mut out = Vec::new();
    let mut vm = Vm::new(VARS.clone(), ARRAYS.clone());

    match vm.run(&mut out, &ir_opt) {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to run '{}' {:?}", filename, e);
        },
    }

    let out_s = String::from_utf8(out).expect(&format!("Output of '{}' is not utf8", filename));
    assert_eq!(out_s, output, "{}\n{:#?}\n{:#?}\n{:#?}", filename, &ir_opt, nodes, FULL_OPT);
}


#[test]
fn positive_snippets() {
    let mut stdout = stdout().lock();

    for path in read_dir("examples").unwrap() {
        let pathb = path.unwrap().path();
        let path: &str = pathb.to_str().unwrap();

        if !path.contains(".snapshot") {
            let code = read_to_string(path).unwrap();
            
            let snapshot = path.replace(".pipa", ".snapshot");
            let err_msg = format!("Snapshot for '{}' has not been found. Create '{}'", path, snapshot);
            let out = read_to_string(snapshot).expect(&err_msg);

            test_str(&mut stdout, path, &code, &out);
        }
    }
    
}

// property testing

#[test]
fn unused_macro_produce_nothing() {
    let mut stdout = stdout().lock();
    let code = "{{ @add_hello \"value not used\" }}";

    test_str(&mut stdout, "*.pipa", code, "");
}


#[test]
fn pipe_into_empty_string_produce_nothing() {
    let mut stdout = stdout().lock();
    let code = "{{ 42 | \"$(_) - super value\" | \"$(_) - lets do more\" | \"\" }}";

    test_str(&mut stdout, "*.pipa", code, "");
}


#[test]
fn pipe_of_value_produce_value() {
    let mut stdout = stdout().lock();
    let code = "{{ \"😃\" | \"$(_)\" }}";

    test_str(&mut stdout, "*.pipa", code, "😃");
}


#[test]
fn multi_pipe_of_value_produce_value() {
    let mut stdout = stdout().lock();
    let code = "{{ \"😃\" | \"$(_)\" | \"$(_)\" | \"$(_)\" | \"$(_)\" | \"$(_)\" }}";

    test_str(&mut stdout, "*.pipa", code, "😃");
}


#[test]
fn empty_range_should_produce_nothing() {
    let mut stdout = stdout().lock();
    let code = "{{ PHONES[0:0] | \"$(_item_)$(_index_)\" }}";

    test_str(&mut stdout, "*.pipa", code, "");
}


#[test]
fn comments_should_produce_nothing() {
    let mut stdout = stdout().lock();
    let code = "{{ # PHONES[:] | \"$(_item_)$(_index_)\" }}";

    test_str(&mut stdout, "*.pipa", code, "");
}


#[test]
fn multi_empty_lines_produce_nothing() {
    let mut stdout = stdout().lock();
    let code = "{{ \"\" \"\" \"\" \"\" \"\" \"\" \"\" \"\" }}";

    test_str(&mut stdout, "*.pipa", code, "");
}
