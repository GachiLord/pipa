use pipa::vm::Vm;
use pipa::syntax::ast;
use pipa::ir::gen_ir;
use pipa::analysis::NO_OPT;
use pipa::utils::{VARS, ARRAYS};
use std::fs::{exists, read_to_string, read_dir, write};
use std::io::stdout;


fn main() {
    let mut f = stdout().lock();
    
    for path in read_dir("examples").unwrap() {

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

        let ir = match gen_ir(&code, nodes, NO_OPT) {
            Ok(ir) => ir,
            Err(e) => {
                e.write_message(&mut f, path, &code).unwrap();
                panic!();
            },
        };

        
        let mut out = Vec::new();
        let mut vm = Vm::new(&VARS, &ARRAYS);

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
