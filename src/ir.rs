use std::fmt;
use std::collections::{HashSet};
use std::io::Write;
use crate::syntax::{Node, TokenType, InnerNode};
use crate::error::CompileError;
use crate::analysis::{evaluate_expr, OptOptions};

#[derive(PartialEq, Debug, Clone)]
pub enum Op {
    PutStr {
        value: Box<str>,
    },
    Flush,
    Collapse,
    PutName {
        start: Option<usize>,
        end: Option<usize>,
        name: Box<str>,
    },
    SetCounter {
        value: usize,
    },
    IncCounter,
    LoadCounter,
    CmpCounterLessJmp {
        op_index: usize,
        value: Option<usize>,
        name: Box<str>,
    },
    CmpArrayEmptyJmp {
        op_index: usize,
        start: Option<usize>,
        end: Option<usize>,
        name: Box<str>,
    },
    LoadArrayItem {
        name: Box<str>,
    },
    PutScopeVar {
        name: Box<str>,
    },
    DestroyScope,
}


impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::PutStr { .. } => {
                write!(f, "PutStr")
            },
            Op::Flush => {
                write!(f, "Flush")
            },
            Op::Collapse => {
                write!(f, "Collapse")
            },
            Op::PutName { start, end, name } => {
                write!(f, "PutName {}[{}:{}]", name, start.unwrap_or_default(), end.unwrap_or_default())
            },
            Op::SetCounter { value } => {
                write!(f, "SetCounter {}", value)
            },
            Op::IncCounter => {
                write!(f, "IncCounter")
            },
            Op::LoadCounter => {
                write!(f, "LoadCounter")
            },
            Op::CmpCounterLessJmp { op_index, value, name } => {
                write!(f, "CmpCounterLessJmp {} {} {}", op_index, value.unwrap_or_default(), name)
            }
            Op::CmpArrayEmptyJmp { op_index, start, end, name } => {
                write!(f, "CmpArrayEmptyJmp {} {} {} {}", op_index, start.unwrap_or_default(), end.unwrap_or_default(), name)
            },
            Op::LoadArrayItem { name } => {
                write!(f, "LoadArrayItem {}", name)
            },
            Op::PutScopeVar { name } => {
                write!(f, "PutScopeVar {}", name)
            },
            Op::DestroyScope => {
                write!(f, "DestroyScope")
            },
        }
    }
}

#[derive(Copy, PartialEq, Debug, Clone)]
pub enum Type {
    String, 
    Int,
    Array,
    Literal,
    Name,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Type::String => "'String'",
            Type::Int => "'Int'",
            Type::Array => "'Array'",
            Type::Literal => "'Literal'",
            Type::Name => "'Name'",
        };

        write!(f, "{}", s)
    }
}

pub fn is_name_reserved(name: &str) -> bool {
    name.starts_with("_")
}

pub fn is_name_array(name: &str) -> bool {
    name.chars().all(|v| {
        let is_int = matches!(v, '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9');

        v.is_uppercase() || is_int || v == '_' 
    }) && name.len() > 1
}

fn in_scope(first_char: usize, name: &str, scope: &mut HashSet<Box<str>>) -> Result<(), CompileError> {
    if !scope.contains(name) && is_name_reserved(&name) {
        return Err(CompileError::new_undefined_var(first_char, name.to_string()));
    }

    Ok(())
}

fn gen_primitive_ir(code: &str, node: &Node, scope: &mut HashSet<Box<str>>, ops: &mut Vec<Op>) -> Result<(), CompileError> {
    match *node.inner {
        InnerNode::Literal { .. } => {
            let value = node.as_escaped_string(code, &[TokenType::CodeBegin, TokenType::CodeEnd]).into();
            ops.push(Op::PutStr { value });
        },
        InnerNode::Name { start, end } => {
            let name: Box<str> = node.as_str(code).into();

            in_scope(node.first_char, &name, scope)?;

            ops.push(Op::PutName { name, start, end });
        },
        InnerNode::Int { value } => {
            ops.push(Op::PutStr { value: value.to_string().into() });
        },
        _ => unreachable!("{:#?}", node),
    }

    Ok(())
}

fn gen_string_ir(code: &str, children: &Vec<Node>, scope: &mut HashSet<Box<str>>, ops: &mut Vec<Op>) -> Result<(), CompileError> {
    for n in children {
        gen_primitive_ir(code, n, scope, ops)?;
    }
    ops.push(Op::Collapse);

    Ok(())
}

fn gen_expr_ir(code: &str, mut node: Node, scope: &mut HashSet<Box<str>>, ops: &mut Vec<Op>) -> Result<(), CompileError> {

    loop {
        match *node.inner {
            InnerNode::String { ref children } => {
                gen_string_ir(code, children, scope, ops)?;
            },
            InnerNode::Name { .. } => {
                gen_primitive_ir(code, &node, scope, ops)?;
            },
            InnerNode::Int { .. } => {
                gen_primitive_ir(code, &node, scope, ops)?;
            },
            _ => unreachable!(),
        }

        if let Some(child) = node.children.pop() {
            scope.insert("_".into());
            ops.push(Op::PutScopeVar{ name: "_".into() });

            node = child;
        } else {
            break;
        }
    }

    Ok(())
}

pub fn gen_ir(code: &str, ast: Vec<Node>, opt: OptOptions) -> Result<Vec<Op>, CompileError> {
    let mut scope = HashSet::new();
    let mut ops = Vec::with_capacity(ast.len());
    let mut iter = ast.into_iter();

    while let Some(mut node) = iter.next() {
        match *node.inner {
            InnerNode::Literal => {
                gen_primitive_ir(code, &node, &mut scope, &mut ops)?;
                ops.push(Op::Flush);
            },
            InnerNode::String { .. } | InnerNode::Int { .. } | InnerNode::Name { .. } => {
                if opt.string_evaluation {
                    match evaluate_expr(node, code) {
                        Some(n) => node = n,
                        None => {
                            scope.clear();
                            continue;
                        }
                    }
                }

                gen_expr_ir(code, node, &mut scope, &mut ops)?;
                ops.push(Op::Flush);

                scope.clear();
            },
            InnerNode::Array { name, start, end } => {
                let child = match opt.string_evaluation {
                    true => {
                        let node = evaluate_expr(node.children.pop().expect("Should be handled during syntax analysis"), code);

                        match node {
                            Some(n) => n,
                            None => {
                                scope.clear();
                                continue;
                            }
                        }
                    },
                    false => {
                        node.children.pop().expect("Should be handled during syntax analysis")
                    }
                };

                ops.push(Op::SetCounter { value: start.unwrap_or(0) });
                let op_index_begin = ops.len();
                ops.push(Op::CmpArrayEmptyJmp{ op_index: 0, start, end, name: name.clone().into() });
                ops.push(Op::LoadArrayItem { name: name.clone().into() });

                ops.push(Op::PutScopeVar { name: "_item_".into() });
                scope.insert("_item_".into());

                ops.push(Op::LoadCounter);
                ops.push(Op::PutScopeVar { name: "_index_".into() });
                scope.insert("_index_".into());

                gen_expr_ir(code, child, &mut scope, &mut ops)?;

                ops.push(Op::Flush);
                ops.push(Op::DestroyScope);
                scope.clear();

                ops.push(Op::IncCounter);

                let op_index_end = ops.len();
                if let Op::CmpArrayEmptyJmp { op_index, .. } = &mut ops[op_index_begin] {
                    *op_index = op_index_end;
                }

                ops.push(Op::CmpCounterLessJmp { name: name, value: end, op_index: op_index_begin });

            },
        }
    }

    Ok(ops)
}

pub fn dump_ir(w: &mut impl Write, ir: &Vec<Op>) -> std::io::Result<()> {
    write!(w, "IR:\n")?;
    for i in 0..ir.len() {
        write!(w, "{}: {}\n", i, &ir[i])?;
    }
    Ok(())
}
