// include the file to avoid multy-crate bullshit
#![allow(unused)]
include!("../utils.rs");

use pipa::vm::Vm;
use pipa::syntax::{Node, TokenType, ast};
use pipa::ir::{Op, gen_ir, dump_ir};
use pipa::analysis::NO_OPT;
use std::fs::{exists, read_to_string, read_dir, write};
use std::io::stdout;
use std::env;


fn main() {
    let mut f = stdout();
    let mut args = env::args();
    let path = &args.nth(1).unwrap();
    let code = read_to_string(path).unwrap();

    let nodes = match ast(&code) {
        Ok(n) => n,
        Err(e) => {
            e.write_message(&mut f, path, &code).unwrap();
            panic!();
        }
    };

    let ir = match gen_ir(&code, nodes.clone(), NO_OPT) {
        Ok(ir) => ir,
        Err(e) => {
            e.write_message(&mut f, path, &code).unwrap();

            println!("{}", code);
            println!("-------------------------------------------");
            println!("{:#?}\n", nodes);

            panic!();
        }
    };

    println!("{}", code);
    println!("-------------------------------------------");
    println!("{:#?}\n\n{:#?}", nodes, ir);
}
