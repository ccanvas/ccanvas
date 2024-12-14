# Connection

Connections are to be stored in a
```rs
static mut CONNECTIONS: HashMap<usize, Connection>
```
to allow direct access: recursive structures, locks are to be avoided at all cost.

Only one thread will be modify this HashMap, and so unsafe statics can be safely ignored.

## Definition

```rs
struct Connection {
    parent: usize,
    data: HashMap<Vec<u8>, Vec<u8>>,
    children: HashSet<usize>,

    client: Option<UnixStream>,
    server: UnixListener,
}
```

- Consider the connections to be a disconnected tree. If `parent` == `self.id`, then `self` is a root node.
- Cycles are not allowed, a node must not be ancestor of itself.

> If `self.client` is none, then the component is *detached* and cannot be reached.

## Operations

### Messaging

```hs
-- sends byte vector to all children connections
message :: Vec<u8> -> ()
```

### Management

```hs
-- set parent space of an existing connection
setParent :: usize -> ()

-- get ID of parent
parentID :: usize

-- get connection instance of parent
parent :: &Connection
```
