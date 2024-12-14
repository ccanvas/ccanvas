# Message thread

The message thread is responsible for sending outbound messages, supporting the operation `send`.

```hs
send :: Int -> Bytes -> ()
```

## Implementation

```rs
static (tx, rx) = mpsc::channel();

for (ids, bytes) in recv {
    ids.for_each(|| connections.get(id).sender.send(bytes))
}
```
