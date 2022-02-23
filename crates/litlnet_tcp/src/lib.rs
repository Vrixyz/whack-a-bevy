pub use litlnet_trait::Communication;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Deserializer;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub struct TcpClient {
    stream: TcpStream,
}

impl TcpClient {
    pub fn connect(remote_addr: &str) -> Result<Self, std::io::Error> {
        let stream = TcpStream::connect(remote_addr)?;
        stream.set_nonblocking(true)?;
        Ok(Self { stream })
    }
    pub fn from_stream(stream: TcpStream) -> Result<Self, std::io::Error> {
        stream.set_nonblocking(true)?;
        Ok(Self { stream })
    }
}

impl Communication for TcpClient {
    fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<Vec<T>>, std::io::Error> {
        let mut buff = vec![0; 256];
        match self.stream.read(&mut buff) {
            Ok(amt) => {
                let de = Deserializer::from_slice(&buff[0..amt]);

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
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()> {
        let mut buf = Vec::new();
        message.serialize(&mut serde_json::Serializer::new(&mut buf))?;
        self.stream.write_all(buf.as_slice())?;
        Ok(())
    }
}
