# tokio broadcast example
An example server built on top of Tokio to broadcast data received from each TCP
client. The server uses tokio-core's `TcpStream`s to send to and receive
from clients and `channel`s to communicate internally.
