# msg

Request a connection to another component.

- Target: `CCANVAS_HOME/TARGET_ID/client.sock`
- Response: varies

```json
{
    "t": "msg",
    "s": usize?, // source ID
    "d": [Binary] // payload
}
```
