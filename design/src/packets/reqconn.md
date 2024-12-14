# reqconn

Request a connection to server.

- Target: `CCANVAS_HOME/PARENT_ID/server.sock`
- Response: `apprconn`, `rejconn`

```json
{
    "t": "reqconn",
    "id": usize, // module id
    "d": HashMap<[u8], [u8]>, // connection info
}
```