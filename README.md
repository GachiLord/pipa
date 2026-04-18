# pipa

```
Pipa is a statically-typed template language written in Rust.

This section is gonna be written as is.
Text inside of {{ }} will be evaluated and inserted in the final output.

The philosophy is "you can(not) fail".
That means, if code is parsed and validated, it must not fail at runtime.
The '(not)' part stands for OOM, OS or hardware issues. You're a not safe after all.

{{
  # This is a comment, so the output won't be changed
  However this is still a valid code
  # The above code works because lowercase ASCII sequences separated by spaces
  # are treated as constant names. Their values will be inserted into the output if present.

  # Also you can use integers in names 
  const_example_0
  const_example_1

  # We have unsigned integers, but no arithmetic operations :(
  # written as is
  69

  # Pipe operations
  # Every value can be piped into a string meaning its contents(depending on type)
  # will be inserted using string interpolation.
  # For example:
  69 | "Value: $(_)"
  # The code above will output 'Value: 69'
  # You can pipe as many times as you want:
  69 | "Value: $(_)" | "$(_)!" | "$(_)!???"
  
  # Arrays
  # It is special type that holds 2 values when piped.
  # Moreover it is the only type that cannot be printed without piping.
  # Arrays are written as uppercase ASCII sequences with brakets at the end.
  ARGS[:] | "$(_item_)$(_index_)"
  # You can provide optional boundaries, which will limit the elements taken
  ARGS[2:] | "$(_item_)$(_index_)"
  ARGS[:5] | "$(_item_)$(_index_)"
  ARGS[2:5] | "$(_item_)$(_index_)"

  # Macros start with '@' letter and ASCII sequence defining its name.
  @const_msg "This is a macro that returns string"
  # Macro can contain pipes but not another macros
  @add_hello "$(_)" | "New value: $(_)"

  # To use a macro, you should expand it
  ?const_msg 
  # Outputs 'This is a macro that returns string'

  # Macro in pipe 
  ARGS[0:69] | "<h1>$(_item_)$(_index_)<h2>" | ?add_hello | "$(_)\n\t"

}}
```

## Usage

Try pipa in [Playground](https://gachilord.github.io/pipa-playground/)

Use pipa as a standalone executable

```bash
cargo install
# constants are mapped to environment vars
lang="pipa" FILES=$(ls) pipa 'Hello from {{ lang }}. Heres your files: {{ "\n\n" FILES[:] | "* $(_item_)\n" }}'
```

Or embed it into your project
```rust
use std::collections::BTreeMap;
use pipa::syntax::ast;
use pipa::vm::Vm;
use pipa::ir::gen_ir;
use pipa::analysis::FULL_OPT;
use pipa::vm::{ArrayVars, StringVars};


fn main() {
    let mut output = Vec::new();

    // gen ir
    let code = r#"Hello from {{ lang }}. Heres your files: {{ "\n\n" FILES[:] | "* $(_item_)\n" }}"#;

    let nodes = ast(&code).unwrap();
    let ir = gen_ir(&code, nodes, FULL_OPT).unwrap();

    // define constants
    let constants: StringVars = BTreeMap::from([
        ("lang".into(), "pipa".into())
    ]);
    let arrays: ArrayVars = BTreeMap::from([
        ("FILES".into(), vec!["one.txt".into(), "two.txt".into()])
    ]);
    
    // run
    let mut vm = Vm::new(constants, arrays);
    vm.run(&mut output, &ir).unwrap();
}
```

## IR

* `PutStr` ( value ) – push value onto the stack  
* `Flush` – output all values on the stack to the output buffer  
* `Collapse` – concatenate all values on the stack into a single string  
* `PutName` ( start, end, name ) – push a constant onto the stack using the provided range in the arguments  
* `SetCounter` ( value ) – set counter to value  
* `IncCounter` – increment counter  
* `LoadCounter` – push counter onto the stack  
* `CmpCounterLessJmp` ( op_index, value, name ) – if counter is less than value, or, in its absence, the length of the name array, then set pc to op_index  
* `CmpArrayEmptyJmp` ( op_index, start, end, name ) – if the name array within the bounds start and end contains no elements, then set pc to op_index  
* `LoadArrayItem` ( name ) – push the element of the name array at index counter onto the stack  
* `PutScopeVar` ( name ) – remove the top element from the stack and assign its value to the scope variable name  
* `DestroyScope` – remove all variables from the scope

## Optimizations

* String evaluation

    Input: `"$(_index_): $(_item_)" | "<li>$(_)</li>"`

    Output: `"<li>$(_index_): $(_item_)</li>"`

* Batching of Flush op

    Input:    
    `PutStr`  
    `Flush` - unnecessary  
    `PutStr`  
    `PutName`  
    `PutStr`  
    `PutName`  
    `PutStr`  
    `Collapse` - unnecessary   
    `Flush`  

    Output:    
    `PutStr`  
    `PutStr`  
    `PutName`  
    `PutStr`  
    `PutName`  
    `PutStr`  
    `Flush`    

* Elimination of unused PutScopeVar op

    Input:  
    `LoadArrayItem LIST`  
    `PutScopeVar _item_`  
    `LoadCounter` - unused     
    `PutScopeVar _index_` - unused     
    `PutStr`  
    `PutName _item_[0:0]`  
    `Collapse`  
    `Flush`  
    `DestroyScope`

    Output:  
    `LoadArrayItem LIST`  
    `PutScopeVar _item_`  
    `PutStr`  
    `PutName _item_[0:0]`  
    `Collapse`  
    `Flush`  
    `DestroyScope`

## Todo

1. ~Use proper AST~
1. ~Add optimizations on AST and IR levels~
1. _Currently not planned_: add `include` directive(maybe with some multi-threading)  
Requires rewrite of IR and AST generation and how vars are handled. Also functions should be introduced.
1. ~CLI-interpreter~
1. _Currently not planned_: compiler  
The language has some features that are difficult to implement in assembly, QBE or WASM.
    * **UTF-8 support** - the main problem I haven't figured out how to solve. I planned to link the binary with some library, or compile a subset of `vm` module that handles graphemes in text.
    * **IR** - it's bloated and doesn't map to low-level instructions well. This is a design flaw that can be resolved in future, though unlikely.  
    Again, I tried to solve this by creating sort of `runtime` for the language similiar to dart, but it turned out that most of the actual code to be run was located in `runtime`(the QBE prototype was calling functions and updating counter LOL) which is not what I wanted to accomplish.


1. Stabilize API

