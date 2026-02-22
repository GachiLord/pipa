// include the file to avoid multy-crate bullshit
#![allow(unused)]
include!("../utils.rs");

use pipa::vm::Vm;
use pipa::syntax::{Node, TokenType, ast};
use pipa::ir::{Op, gen_ir, dump_ir};
use std::fs::{exists, read_to_string, read_dir, write};
use std::io::stdout;


fn main() {
    let mut f = stdout().lock();
    
    for path in read_dir("../examples").unwrap() {

        let pathb = path.unwrap().path();
        let path: &str = pathb.to_str().unwrap();
        
        if path.contains(".snapshot") {
            continue;
        }

        let code = read_to_string(path).unwrap();
        let snapshot = path.replace(".pipa", ".snapshot");

        if exists(&snapshot).unwrap() {
            println!("Snapshot '{}' exists. Skipping", snapshot);
            continue;
        }

        // gen snapshot
        let nodes = match ast(&code) {
            Ok(n) => n,
            Err(e) => {
                e.write_message(&mut f, path, &code).unwrap();
                panic!();
            }
        };

        let ir = match gen_ir(&code, nodes) {
            Ok(ir) => ir,
            Err(e) => {
                e.write_message(&mut f, path, &code).unwrap();
                panic!();
            },
        };

        
        let mut out = Vec::new();
        let mut vm = Vm::new(VARS.clone(), ARRAYS.clone());

        match vm.run(&mut out, &ir) {
            Ok(_) => {},
            Err(e) => {
                panic!("Failed to run '{}' {:?}", path, e);
            },
        }
        // write to file

        write(&snapshot, &out).unwrap();
        println!("Generated snapshot '{}'", snapshot);
    }
}
