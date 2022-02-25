This is WIP to enable browser support.

For the moment it doesn't use `litl_*` crates nor bevy, but it communicates with example_server by leveraging https://rustwasm.github.io/wasm-bindgen/examples/websockets.html 

# Other work

https://github.com/rerun-io/ewebsock uses a channel to communicate from web-sys websocket to user application, it's cleaner than unsafe global variable!