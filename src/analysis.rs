use std::mem;
use crate::syntax::{InnerNode, Node};

pub const NO_OPT: OptOptions = OptOptions{ string_evaluation: false };
pub const FULL_OPT: OptOptions = OptOptions{ string_evaluation: true };

#[derive(Clone, Copy, Default, Debug)]
pub struct OptOptions {
    pub string_evaluation: bool,
    // TODO Flush batching
    // TODO PutScopeVar
    // TODO Deadcode elimination
}

pub fn evaluate_expr(parent: Node, code: &str) -> Option<Node> {
    if parent.children.is_empty() {
        if let InnerNode::String { ref children } = *parent.inner {
            if children.len() == 0 {
                return None;
            }
        }

        return Some(parent);
    }

    let mut tail = parent;
    let mut parent_expr = Vec::new();
    let mut child_expr = Vec::new();
    // expand parent
    match *tail.inner {
        InnerNode::String { children } => {
            parent_expr = children;
            tail = tail.children.pop().unwrap();
        },
        InnerNode::Int { .. } | InnerNode::Name { .. } => {
            let children = tail.children.pop().unwrap();
            parent_expr.push(tail);
            tail = children;
        },
        InnerNode::Array { .. } | InnerNode::Literal { .. } => {
            unreachable!("This function should not be used with arrays and literals");
        },
    }
    // expand its children
    loop {
        match *tail.inner {
            InnerNode::String { children, .. } => {

                for child in children {

                    match *child.inner {
                        InnerNode::Literal => {
                            child_expr.push(child);
                        },
                        InnerNode::Name { .. } => {
                            if child.as_str(code) == "_" {
                                // we need to copy because there can be more than one '_'
                                // Example: 69 | "$(_)     $(_)"
                                child_expr.extend_from_slice(&parent_expr[..]);
                            } else {
                                child_expr.push(child);
                            }
                        },
                        InnerNode::String { .. } | InnerNode::Int { .. } | InnerNode::Array { .. } => {
                            unreachable!("Should be handled during ast building");
                        },
                    }
                }

                parent_expr.clear();
                mem::swap(&mut child_expr, &mut parent_expr);

                match tail.children.pop() {
                    Some(child) => {
                        tail = child;
                    },
                    None => {
                        break;
                    }
                }
            },
            InnerNode::Array { .. } | InnerNode::Literal { .. } | InnerNode::Int { .. } | InnerNode::Name { .. } => {
                unreachable!("Should be handled during ast building");
            },
        }
    }

    if parent_expr.len() == 0 {
        return None;
    }

    Some(Node::new(
            parent_expr[0].first_char,
            parent_expr[0].end_char,
            InnerNode::String {children: parent_expr},
            Vec::new())
        )
}

mod test {
    use crate::syntax::{ast, InnerNode, Node};
    use crate::analysis::{evaluate_expr};

    #[test]
    fn empty_string_evaluation() {
        let code = "{{ \"\" }}";
        let nodes = ast(&code).unwrap();

        let evaluated = evaluate_expr(nodes[0].clone(), code);

        assert_eq!(None, evaluated);
    }

    #[test]
    fn string_pipe_evaluation() {
        let code = "{{ \"value\" | \"$(_)\" }}";
        let nodes = ast(&code).unwrap();

        let evaluated = evaluate_expr(nodes[0].clone(), code);
        let mut expected = None;

        if let InnerNode::String { ref children } = *nodes[0].inner {

            let n = Node::new(
                children[0].first_char,
                children[0].end_char,
                InnerNode::String {children: children.to_vec()},
                Vec::new()
            );

            expected = Some(n);
        }

        assert_eq!(expected, evaluated);
    }



    #[test]
    fn string_constants_pipe_evaluation() {
        let code = "{{ \"$(first)value$(second)\" | \"$(_)\" }}";
        let nodes = ast(&code).unwrap();

        let evaluated = evaluate_expr(nodes[0].clone(), code);
        let mut expected = None;

        if let InnerNode::String { ref children } = *nodes[0].inner {

            let n = Node::new(
                children[0].first_char,
                children[0].end_char,
                InnerNode::String {children: children.to_vec()},
                Vec::new()
            );

            expected = Some(n);
        }

        assert_eq!(expected, evaluated);
    }


    #[test]
    fn int_pipe_evaluation() {
        let code = "{{ 69 | \"$(_)\" }}";
        let nodes = ast(&code).unwrap();

        let evaluated = evaluate_expr(nodes[0].clone(), code);
        let mut expected = None;

        if let InnerNode::Int { value } = *nodes[0].inner {
            let int = Node::new(nodes[0].first_char, nodes[0].end_char, InnerNode::Int{ value }, Vec::new());

            let n = Node::new(
                nodes[0].first_char,
                nodes[0].end_char,
                InnerNode::String {children: vec![int] },
                Vec::new()
            );

            expected = Some(n);
        }

        assert_eq!(expected, evaluated);
    }


    #[test]
    fn name_pipe_evaluation() {
        let code = "{{ first | \"$(_)\" }}";
        let nodes = ast(&code).unwrap();

        let evaluated = evaluate_expr(nodes[0].clone(), code);
        let mut expected = None;

        if let InnerNode::Name { start, end } = *nodes[0].inner {
            let name = Node::new(nodes[0].first_char, nodes[0].end_char, InnerNode::Name{ start, end }, Vec::new());

            let n = Node::new(
                nodes[0].first_char,
                nodes[0].end_char,
                InnerNode::String {children: vec![name] },
                Vec::new()
            );

            expected = Some(n);
        }

        assert_eq!(expected, evaluated);
    }
    
}
