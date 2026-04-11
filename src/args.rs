use std::env::Args;
use crate::analysis::{OptOptions, NO_OPT, FULL_OPT};

#[derive(Debug)]
pub struct ArgOptions {
    pub help: bool,
    pub opt: OptOptions,    
    pub separator: String,
    pub file: String,
}

pub const USAGE: &'static str = 
"Usage: pipa [options] script.pipa

Options:
    --help                      print this message
    --no-opt                    disable all optimizations enabled by default and specified before this option
    --fstring_evaluation        enable string evaluation optimization
    --fconstant_evaluation      enable constant evaluation optimization
    --sep                       specify separator for array constants(default: '\\n')
";

pub fn parse(args: Args) -> ArgOptions {
    let mut opts = ArgOptions {
        help: false,
        opt: FULL_OPT,
        separator: "\n".into(),
        file: "".into(),
    };

    let mut iter = args.peekable();

    // first arg is executable
    let _ = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--no-opt" => opts.opt = NO_OPT,
            "--help" => opts.help = true,
            "--fstring_evaluation" => opts.opt.string_evaluation = true,
            "--fconstant_evaluation" => opts.opt.constant_evaluation = true,
            "--sep" => {
                match iter.next() {
                    Some(sep) => opts.separator = sep,
                    None => opts.help = true,
                }
            },
            _ => opts.file = arg,
        }
    }

    opts
}

