# Connection

## Prerequisite

- Directory `MODULE_HOME=CCANVAS_HOME/ID` exists.
- Socket `MODULE_HOME/client.sock` exists.

## Instructions

1. Generate a unique `usize` as identifier.
2. Send a `reqconn` packet to `CCANVAS_HOME/PARENT_ID/server.sock`.

## Responses

### Connection approved

Connection is approved if:
- No other module with the same identifier exists.

The server would:
- Create a listener socket at `MODULE_HOME/server.sock` accepting requests.
- Send a `apprconn` packet to `MODULE_HOME/client.sock`.

> If `client.sock` is not present on creation, then a *detached connection* is created.

### Connection rejected

Connection is rejected if:
- There exist another module with the same identifier.

The server would:
- Send a `rejconn` packet to `MODULE_HOME/client.sock`, if the module with the same identifier is a space.
- Do nothing, if the module with the same identifier is a connection.

> Note that it is the client's responsibility to clean up files appropriately.
