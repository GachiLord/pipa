# pipa

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

