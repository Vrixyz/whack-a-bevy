# Given that:

- Making multiplayer games is hard: tcp, udp, websocket, connection, errors, reconnection, latency...

# Then this library

- Helps you set up a multiplayer game where you want to "send data", and "receive data".

## Modularity

- **a common core** trait in `litlnet_trait`
- **base implementation** of that *common core* for popular protocols using libraries in `litlnet_tcp`, `litlnet_websocket`
- **server implementation** leveraging the *base implementation* in `litlnet_tcp_server`, `litlnet_websocket_server`
- **bevy bridges**: compatible with any *base implementation* in `litlnet_client_bevy` and `litlnet_server_bevy`
- **short examples** in `example_*`

# (very) WIP

- support browser: see ./crates/example_client_web/Readme.md
