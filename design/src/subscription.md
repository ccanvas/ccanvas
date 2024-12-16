# Subscription

A connection can subscribe to events from the server or other components.

Subscriptions are to be stored in a static map, only accessed by the processor thread.

```rs
static SUBSCRIPTIONS: HashMap<Channel, HashSet<usize>>
```
