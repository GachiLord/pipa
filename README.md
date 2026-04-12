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

