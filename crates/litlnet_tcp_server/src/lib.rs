use litlnet_tcp::{Communication, TcpClient};
use litlnet_trait::{ClientId, Server};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::net::TcpListener;

struct Client {
    com: TcpClient,
}

pub struct ComServer {
    listener: TcpListener,
    clients: HashMap<ClientId, Client>,
    next_available_id: ClientId,
    to_be_removed: Vec<ClientId>,
}

impl ComServer {
    pub fn iter(&self) -> impl Iterator<Item = &ClientId> + '_ {
        self.clients.keys()
    }
}

impl Server for ComServer {
    fn bind(addr: &str) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            clients: HashMap::new(),
            next_available_id: ClientId(usize::MIN),
            to_be_removed: vec![],
        })
    }
    fn accept_connections(&mut self) {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client = Client {
                        com: TcpClient::from_stream(stream).expect("failed to create ComClient"),
                    };
                    self.clients.insert(self.next_available_id, client);
                    self.next_available_id.0 = self.next_available_id.0.wrapping_add(1);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
    fn receive_all<T: DeserializeOwned>(
        &mut self,
        mut read_callback: impl FnMut(ClientId, Vec<T>),
    ) {
        for (id, client) in self.clients.iter_mut() {
            match client.com.receive::<T>() {
                Ok(Some(data)) => {
                    read_callback(*id, data);
                }
                Ok(None) => {}
                Err(e) => {
                    dbg!(e);
                    self.to_be_removed.push(*id);
                }
            }
        }
        for to_clean in &self.to_be_removed {
            self.clients.remove(to_clean);
        }
    }
    fn send<T: Serialize>(&mut self, client_id: &ClientId, data: &T) {
        match self.clients.get_mut(client_id) {
            Some(client) => match client.com.send::<T>(data) {
                Ok(()) => {}
                Err(e) => {
                    dbg!(e);
                    self.to_be_removed.push(*client_id);
                }
            },
            None => {}
        }
    }
}
