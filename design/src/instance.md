# Instance

Instance holds information about the current running instance of ccanvas, such as.

- Instance ID
- Instance path

## Initialisation

Instance is responsible for initialising static variables.

```rs
static instance_id = random();
static instance_path = "/tmp/ccanvas/ID"

ConnectionThread::spawn();

Self::path_create();
Connection::init();
```
