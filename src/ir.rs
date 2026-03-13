use std::fmt;
use std::collections::{HashSet};
use std::io::Write;
use crate::syntax::{Node, TokenType, InnerNode};
use crate::error::CompileError;

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
    name.chars().all(|v| v.is_uppercase())
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
        InnerNode::Name { .. } => {
            let name: Box<str> = node.as_str(code).into();

            in_scope(node.first_char, &name, scope)?;

            ops.push(Op::PutName { start: None, end: None, name });
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

pub fn gen_ir(code: &str, ast: Vec<Node>) -> Result<Vec<Op>, CompileError> {
    let mut ops = Vec::with_capacity(ast.len());
    let mut scope = HashSet::new();
    let mut iter = ast.into_iter();

    while let Some(mut node) = iter.next() {
        match *node.inner {
            InnerNode::Literal => {
                gen_primitive_ir(code, &node, &mut scope, &mut ops)?;
                ops.push(Op::Flush);
            },
            InnerNode::String { .. } | InnerNode::Int { .. } | InnerNode::Name { .. } => {
                gen_expr_ir(code, node, &mut scope, &mut ops)?;
                ops.push(Op::Flush);
                ops.push(Op::DestroyScope);

                scope.clear();
            },
            InnerNode::Array { name, start, end } => {
                // handle name[:]
                if !is_name_array(&name) {
                    in_scope(node.first_char, &name, &mut scope)?;

                    ops.push(Op::PutName { start, end, name });
                    ops.push(Op::Flush);
                    continue
                }
                // handle ARRAY[:]
                ops.push(Op::SetCounter { value: start.unwrap_or(0) });
                let op_index = ops.len();
                ops.push(Op::CmpArrayEmptyJmp{ op_index, start, end, name: name.clone().into() });
                ops.push(Op::LoadArrayItem { name: name.clone().into() });

                ops.push(Op::PutScopeVar { name: "_item_".into() });
                scope.insert("_item_".into());

                ops.push(Op::LoadCounter);
                ops.push(Op::PutScopeVar { name: "_index_".into() });
                scope.insert("_index_".into());
                
                gen_expr_ir(code, node.children.pop().expect("Should be handled during syntax analysis"), &mut scope, &mut ops)?;
                ops.push(Op::Flush);
                ops.push(Op::DestroyScope);
                scope.clear();

                ops.push(Op::IncCounter);
                ops.push(Op::CmpCounterLessJmp { name: name, value: end, op_index });
            },
        }
    }
    /*
    while i < ast.len() {
        let node = &ast[i];
        
        match *node.inner {
            InnerNode::Literal => {
                let value = node.as_escaped_string(code, &[TokenType::CodeBegin, TokenType::CodeEnd]).into();
                ops.push(Op::PutStr { value });
                ops.push(Op::Flush);
            },
            InnerNode::Int { .. } => {
                let value = node.as_str(code).into();
                ops.push(Op::PutStr { value });

                if is_node_pipe(&ast, i + 1) {
                    expect_string(&ast, i + 2)?;
                    ops.push(Op::PutScopeVar { name: "_".into() });
                    scope.insert("_");
                    i += 1;
                } else {
                    ops.push(Op::Flush);
                }
            },
            InnerNode::Name { .. } => {
                let name: Box<str> = node.as_str(code).into();

                // Names cannot be piped
                if is_name_reserved(&name) {
                    return Err(CompileError::new_undefined_var(node.first_char, name.into()));
                }

                ops.push(Op::PutName { name, start: None, end: None });


                if is_node_pipe(&mut ast, i + 1) {
                    expect_string(&ast, i + 2)?;
                    ops.push(Op::PutScopeVar { name: "_".into() });
                    scope.insert("_");
                    i += 1;
                } else {
                    ops.push(Op::Flush);
                }
            },
            InnerNode::Array { ref name, start, end } => {
                if is_name_array(name) {
                    ops.push(Op::SetCounter { value: start.unwrap_or(0) });
                    ops.push(Op::CmpArrayEmptyJmp{ op_index: 0, start, end, name: name.clone().into() });
                    array_start = Some((i, ops.len() - 1));
                    ops.push(Op::LoadArrayItem { name: name.clone().into() });
                    ops.push(Op::PutScopeVar { name: "_item_".into() });
                    ops.push(Op::LoadCounter);
                    ops.push(Op::PutScopeVar { name: "_index_".into() });

                    scope.insert("_item_");
                    scope.insert("_index_");

                    let f = node.first_char;
                    if !is_node_pipe(&mut ast, i + 1) {
                        return Err(CompileError::new_array_pipe(f));
                    }
                    expect_string(&ast, i + 2)?;
                    i += 1;
                } else {
                    ops.push(Op::PutName { name: name.clone(), start, end });

                    if !scope.contains(name.as_ref()) && is_name_reserved(&name) {
                        return Err(CompileError::new_undefined_var(node.first_char, name.to_string()));
                    }

                    if is_node_pipe(&mut ast, i + 1) {
                        expect_string(&ast, i + 2)?;
                        ops.push(Op::PutScopeVar { name: "_".into() });
                        scope.insert("_");
                        i += 1;
                    } else {
                        ops.push(Op::Flush);
                    }
                        
                }

            },
            InnerNode::String { ref children } => {
                for n in children {
                    match *n.inner {
                        InnerNode::Name => {
                            let name = n.as_str(code);
                            
                            if !scope.contains(name) && is_name_reserved(name) {
                                return Err(CompileError::new_undefined_var(n.first_char, name.into()));
                            }

                            ops.push(Op::PutName { name: name.into(), start: None, end: None });
                        },
                        InnerNode::Literal => {
                            let value = n.as_escaped_string(code, &[TokenType::CodeBegin, TokenType::CodeEnd]).into();
                            ops.push(Op::PutStr { value });
                        },
                        _ => unreachable!(),
                    }
                }

                ops.push(Op::Collapse);

                if is_node_pipe(&mut ast, i + 1) {
                    expect_string(&ast, i + 2)?;
                    ops.push(Op::PutScopeVar { name: "_".into() });
                    scope.insert("_");
                    i += 1;
                } else {
                    ops.push(Op::Flush);
                    ops.push(Op::DestroyScope);

                    if let Some((node_i, op_i)) = array_start {
                        let len = ops.len();
                        array_start = None;

                        if let Op::CmpArrayEmptyJmp { op_index, .. } = &mut ops[op_i] {
                            *op_index = len + 1;
                        } else {
                            unreachable!();
                        }

                        if let InnerNode::Array { ref name, end, .. } = *ast[node_i].inner {
                            // push CmpCounterLessJmp op
                            ops.push(Op::IncCounter);
                            ops.push(Op::CmpCounterLessJmp { name: name.clone(), value: end, op_index: op_i } );
                            scope.clear();
                        } else {
                            unreachable!();
                        }
                    }
                }
            },
            InnerNode::NewLine => {},
            InnerNode::Pipe => {
                return Err(CompileError::new_pipe_no_parent(node.first_char));
            },
            InnerNode::Range { .. } => {
                return Err(CompileError::new_invalid_array(node.first_char));
            }
            _ => {
                unreachable!();
            },
        }        

        i += 1;
    }
    */

    Ok(ops)
}

pub fn dump_ir(w: &mut impl Write, ir: &Vec<Op>) -> std::io::Result<()> {
    write!(w, "IR:\n")?;
    for i in 0..ir.len() {
        write!(w, "{}: {}\n", i, &ir[i])?;
    }
    Ok(())
}
