mod utils;

use std::io::{stdout, Write};
use std::fs::{read_to_string, read_dir};
use pipa::ir::gen_ir;
use pipa::syntax::ast;
use pipa::vm::Vm;
use utils::{VARS, ARRAYS};


fn test_str(f: &mut impl Write, filename: &str, code: &str, output: &str) {
    let nodes = match ast(&code) {
        Ok(n) => n,
        Err(e) => {
            e.write_message(f, filename, code).unwrap();
            panic!();
        }
    };

    let ir = match gen_ir(&code, nodes) {
        Ok(ir) => ir,
        Err(e) => {
            e.write_message(f, filename, code).unwrap();
            panic!();
        },
    };

    
    let mut out = Vec::new();
    let mut vm = Vm::new(VARS.clone(), ARRAYS.clone());

    match vm.run(&mut out, &ir) {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to run '{}' {:?}", filename, e);
        },
    }

    // check the difference
    let out = String::from_utf8(out).expect(&format!("Output of '{}' is not utf8", filename));
    assert_eq!(out, output, "{}", filename);    
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
