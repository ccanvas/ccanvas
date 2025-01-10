# Packets

Packets forms the basis of ccanvas.

- A request packet is sent to `CCANVAS_HOME/ccanvas.sock`.
- A response packet is expected from `CCANVAS_HOME/name.sock`

Where the ccanvas server listens to `ccanvas.sock` and the client is listening to client.sock.

- Clients can also directly message each other by sending to the respective `name.sock`.
- If the field `"e"` is included in a packet, then the response packet is also expected to echo whatever is in `"e"`.

All packets are encoded in standard messagepack, if you have found a better encoding format, please let me know.
