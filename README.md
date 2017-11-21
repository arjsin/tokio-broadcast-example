# tokio broadcast example
An example server built on top of Tokio to broadcast data received from each TCP
client. The server uses tokio-core's `TcpStream`s to send to and receive
from clients and `channel`s to communicate internally.
## Usage
To build and run:
```
cargo run
```
This will start server listening on IP `127.0.0.1` and port `8080`. To change it to let say IP `::1`
and port `8000`:
```
cargo run [::1]:8000
```
To change it to let say IP `0.0.0.0` and port `3000`
```
cargo run 0.0.0.0:3000
```
To test the server, netcat or telnet can be used as client. The server will
broadcast the messages received from each client.
