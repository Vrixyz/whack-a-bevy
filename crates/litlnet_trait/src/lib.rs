use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ClientId(pub usize);

impl From<ClientId> for usize {
    fn from(val: ClientId) -> Self {
        val.0
    }
}

pub trait Communication {
    fn receive<T: DeserializeOwned>(&mut self) -> Result<Option<Vec<T>>, std::io::Error>;
    fn send<T: Serialize>(&mut self, message: &T) -> std::io::Result<()>;
}
pub trait Server {
    fn bind(addr: &str) -> Result<Self, std::io::Error>
    where
        Self: Sized;
    fn accept_connections(&mut self);
    fn receive_all<T: DeserializeOwned>(&mut self, read_callback: impl FnMut(ClientId, Vec<T>));
    fn send<T: Serialize>(&mut self, client_id: &ClientId, data: &T);
}
