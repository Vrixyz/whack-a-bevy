pub use litlnet_trait::Communication;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Deserializer;
use std::net::TcpStream;
use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

trait LitlWebsocket {
    fn write_all(&mut self, buf: Vec<u8>) -> Result<(), tungstenite::Error>;
    fn read(&mut self) -> Result<Message, tungstenite::Error>;
}

impl LitlWebsocket for WebSocket<MaybeTlsStream<TcpStream>> {
    fn write_all(&mut self, buf: Vec<u8>) -> Result<(), tungstenite::Error> {
        self.write_message(Message::Binary(buf))
    }

    fn read(&mut self) -> Result<Message, tungstenite::Error> {
        self.read_message()
    }
}
impl LitlWebsocket for WebSocket<TcpStream> {
    fn write_all(&mut self, buf: Vec<u8>) -> Result<(), tungstenite::Error> {
        self.write_message(Message::Binary(buf))
    }

    fn read(&mut self) -> Result<Message, tungstenite::Error> {
        self.read_message()
    }
}

pub struct WebsocketClient {
    websocket: Box<dyn LitlWebsocket + Send + Sync + 'static>,
}

impl WebsocketClient {
    pub fn connect(remote_addr: &str) -> Result<WebsocketClient, std::io::Error> {
        let (mut websocket, _) = match tungstenite::connect(url::Url::parse(remote_addr).unwrap()) {
            Ok(it) => it,
            Err(err) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    dbg!(err),
                ))
            }
        };
        if let MaybeTlsStream::Plain(s) = websocket.get_mut() {
            s.set_nonblocking(true)?
        }

        Ok(WebsocketClient {
            websocket: Box::new(websocket),
        })
    }
    pub fn from_stream(stream: std::net::TcpStream) -> Result<WebsocketClient, std::io::Error> {
        stream.set_nonblocking(true)?;
        match tungstenite::accept(stream) {
            Ok(websocket) => Ok(Self {
                websocket: Box::new(websocket),
            }),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                dbg!(e),
            )),
        }
    }
}

impl Communication for WebsocketClient {
    fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<Vec<T>>, std::io::Error> {
        match self.websocket.read() {
            Ok(Message::Binary(msg)) => {
                let de = Deserializer::from_slice(msg.as_slice());

                // FIXME: a stream deserializer is useful for TCP as we can get multiple messages in the same packet, but websocket handles that so we can just deserialize 1 message.
                let stream = de.into_iter::<T>();
                let mut res = vec![];
                for v in stream {
                    match v {
                        Err(e) => {
                            dbg!(e);
                        }
                        Ok(v) => {
                            res.push(v);
                        }
                    }
                }
                Ok(Some(res))
            }
            Ok(data) => {
                dbg!(data);
                Ok(None)
            }
            Err(tungstenite::Error::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                dbg!(e),
            )),
        }
    }

    fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        let mut buf = Vec::new();
        message.serialize(&mut serde_json::Serializer::new(&mut buf))?;
        match self.websocket.write_all(buf) {
            Ok(it) => it,
            Err(tungstenite::Error::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    dbg!(e),
                ))
            }
        };
        Ok(())
    }
}
