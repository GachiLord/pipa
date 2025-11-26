use unicode_segmentation::UnicodeSegmentation;
use std::collections::BTreeMap;
use std::io::Write;
use crate::ir::{Op, is_name_reserved};

type StringVars = BTreeMap<Box<str>, Box<str>>;
type ArrayVars = BTreeMap<Box<str>, Vec<Box<str>>>;


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VmError {
    EndOfProgram,
    WriteError,
    EmptyStack,
    UndefinedScopeVar,
    ArrayIndexOverflow,
}

#[derive(Debug)]
pub struct Vm {
    counter: usize,
    pc: usize,
    stack: Vec<Box<str>>,
    vars: StringVars,
    scope: StringVars,
    arrays: ArrayVars,
}

impl Vm {
    pub fn new(vars: StringVars, arrays: ArrayVars) -> Self {
        Self {
            counter: 0,
            pc: 0,
            stack: Vec::with_capacity(15),
            vars,
            scope: BTreeMap::new(),
            arrays,
        }
    }


    pub fn clear_state(&mut self) {
        self.pc = 0;
        self.stack.clear();
        self.scope.clear();
    }

    pub fn dump_state(&self) {
        println!("\nVM_STATE counter: {} pc: {} stack_len: {}\n", self.counter, self.pc, self.stack.len());
    }

    fn get_string_var(&self, name: &str) -> Result<&str, VmError> {
        if is_name_reserved(name) {
            let var = self.scope.get(name).ok_or_else(|| VmError::UndefinedScopeVar)?;
            return Ok(var)
        } else {
            match self.vars.get(name) {
                Some(s) => Ok(s),
                None => Ok("")
            }
        }
    }


    fn get_array_var(&self, name: &str) -> &[Box<str>] {
        match self.arrays.get(name) {
            Some(arr) => arr,
            None => &[],
        }
    }

    pub fn run(&mut self, w: &mut impl Write, program: &[Op]) -> Result<(), VmError> {
        loop {
            match self.step(w, program) {
                Err(VmError::EndOfProgram) => break,
                Err(e) => return Err(e),
                Ok(_) => {}
            }
        }

        Ok(())
    }

    pub fn step(&mut self, w: &mut impl Write, program: &[Op]) -> Result<(), VmError> {
        if self.pc >= program.len() {
            return Err(VmError::EndOfProgram);
        }

        match &program[self.pc] {
            Op::PutStr { value } => {
                self.stack.push(value.clone());
            },
            Op::Flush => {
                for s in self.stack.drain(..) {
                    write!(w, "{}", s).map_err(|_| VmError::WriteError)?;
                }
            },
            Op::Collapse => {
                let mut output = String::new();

                for s in self.stack.drain(..) {
                    output.push_str(&s);
                }

                self.stack.push(output.into());

            },
            Op::PutName { start, end, name } => {
                let mut output = String::new();
                let var = self.get_string_var(&name)?;
                let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(var, true).collect();
                let mut start = start.unwrap_or(0);                
                let end = std::cmp::min(var.len(), end.unwrap_or(var.len()));                

                while start < end {
                    output.push_str(graphemes[start]);                    
                    
                    start += 1;
                }

                self.stack.push(output.into());
            },
            Op::SetCounter { value } => {
                self.counter = *value;
            },
            Op::IncCounter => {
                self.counter += 1;
            },
            Op::LoadCounter => {
                self.stack.push(self.counter.to_string().into());
            },
            Op::CmpCounterLessJmp { op_index, value, name } => {
                let len = self.get_array_var(&name).len();

                let value = match value {
                    Some(v) => std::cmp::min(*v, len),
                    None => len,
                };

                if self.counter < value {
                    self.pc = *op_index;
                }
            },
            Op::CmpArrayEmptyJmp { op_index, start, end, name } => {
                let len = self.get_array_var(&name).len();
                let end = std::cmp::min(len, end.unwrap_or(len));

                if start.unwrap_or(0) >= end || end == 0 {
                    self.pc = *op_index;
                }
            },
            Op::LoadArrayItem { name } => {
                let arr = self.get_array_var(&name);
                let item = arr.get(self.counter).ok_or_else(|| VmError::ArrayIndexOverflow)?;
                self.stack.push(item.clone());
            },
            Op::PutScopeVar { name } => {
                let var = self.stack.pop().ok_or_else(|| VmError::EmptyStack)?;
                self.scope.insert(name.clone(), var.into());
            },
            Op::DestroyScope => {
                self.scope.clear();
            },
        }

        self.pc += 1;
        Ok(())
    }
}
