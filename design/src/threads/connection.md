# Connection thread

The connection thread handles all incoming connections, and sends them to the processor thread.

```rs
static poll = Poll::new();
static registry = poll.registry();
let events = Events::with_capacity(1024);

loop {
    poll.poll(&mut events, None);

    for event in &events {
        processor.send((id, bytes))
    }
}
```
