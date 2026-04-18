use pipa::args::{parse, USAGE};
use pipa::vm::{Vm, StringVars, ArrayVars};
use pipa::ir::{is_name_array, gen_ir};
use pipa::syntax::ast;
use std::collections::BTreeMap;
use std::env;
use std::fs::read_to_string;
use std::io::{stdout, Write};


fn main() {
    let mut f = stdout().lock();
    // parse args
    let mut opt = parse(env::args());

    if opt.help || opt.file == "" {
        write!(f, "{}", USAGE).unwrap();
        return;
    }
    // gen ir
    let code = match read_to_string(&opt.file) {
        Ok(c) => c,
        Err(_) => {
            let code = opt.file;
            opt.file = "input.pipa".into();
            code
        },
    };

    let nodes = match ast(&code) {
        Ok(n) => n,
        Err(e) => {
            e.write_message(&mut f, &opt.file, &code).unwrap();
            return;
        }
    };

    let ir = match gen_ir(&code, nodes, opt.opt) {
        Ok(ir) => ir,
        Err(e) => {
            e.write_message(&mut f, &opt.file, &code).unwrap();
            return;
        },
    };

    // collect constants
    let mut constants: StringVars = BTreeMap::new();
    let mut arrays: ArrayVars = BTreeMap::new();
    
    for (key, value) in env::vars() {
        if is_name_array(&key) {
            let values: Vec<Box<str>> = value.split(&opt.separator).map(|v| v.into()).collect();

            arrays.insert(key.into(), values);
            continue;
        }

        constants.insert(key.into(), value.into());
    }

    // run
    let mut vm = Vm::new(&constants, &arrays);

    match vm.run(&mut f, &ir) {
        Ok(_) => {},
        Err(e) => {
            panic!("Vm error: {:?}", e);
        },
    }
}
