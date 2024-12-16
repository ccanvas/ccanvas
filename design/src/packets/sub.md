# sub

Request a subscription to an event.

- Target: `CCANVAS_HOME/ID/server.sock`
- Response: `subbed`

```json
{
    "t": "msg",
    "s": usize?, // source ID
    "d": [Binary] // payload
}
```
