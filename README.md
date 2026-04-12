# pipa

Statically-typed compiled or interpreted programming language for writing templates.

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

## Library API

```c
// libpipa

typedef Pipa_Constant struct {
    const char *key
    const char *value
};

typedef Pipa_Array struct {
    const char *key
    const char **values
};

typedef void* (*Pipa_AllocFunc) (void *ptr, void*);
typedef void* (*Pipa_ReallocFunc) (void *ptr, size_t, void*);
typedef void (*Pipa_FreeFunc) (void *, void*);
typedef void (*Pipa_FlushCallback)(void*, void*);

// compiler output

int pipa_run(
    const Pipa_Constant *constants,
    const Pipa_Array *arrays,
    Pipa_AllocFunc alloc_fn,
    Pipa_ReallocFunc realloc_fn,
    Pipa_FreeFunc free_fn,
    Pipa_FlushCallback flush_cb,
    void *user_data
);
```

## Todo

1. ~Use proper AST~
1. ~Add optimizations on AST and IR levels~
1. _Currently not planned_: add `include` directive(maybe with some multi-threading)  
Requires rewrite of IR and AST generation and how vars are handled. Also functions should be introduced.
1. ~CLI-interpreter~
1. Compiler
    * Add GNU assembler backend for x86-64 Linux, maybe arm64 and other Os'
    * Add QBE backend
    * Add WASM backend
1. Stabilize API

