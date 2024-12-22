# Packets

Packets forms the basis of ccanvas.

- A request packet is sent to `MODULE_HOME/server.sock`.
- A response packet is expected from `MODULE_HOME/client.sock`

Where the ccanvas server listens to `server.sock` and the client is listening to client.sock.

- Clients can also directly message each other by sending to the respective `client.sock`.
- If the field `"e"` is included in a packet, then the response packet is also expected to echo whatever is in `"e"`.

All packets are encoded in standard messagepack, if you have found a better encoding format, please let me know.
