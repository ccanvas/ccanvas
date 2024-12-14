# Module

A module is an independent unit of *something* which can be operated on, both supporting a similar set of operations.

Modules are to be stored in a
```rs
static mut MODULES: HashMap<usize, Module>
```
to allow direct access: recursive structures, locks are to be avoided at all cost.

Only one thread will be modify this HashMap, and so unsafe statics can be safely ignored.

## Definition

```rs
struct Module {
    parent: usize,

    addr: String,
    d: Vec<u8>,
    pid: Option<usize>,
}
```

## Operations

### Messaging

```hs
-- sends byte vector to all submodules
message :: Vec<u8> -> ()
```

### Management

```hs
-- set parent space of an existing module
setParent :: usize -> ()

-- get ID of parent
parentID :: usize

-- get module instance of parent
parent :: &Module
```
