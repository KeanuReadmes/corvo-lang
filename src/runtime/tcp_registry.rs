use crate::{CorvoError, CorvoResult};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Holds live TCP listeners and streams for `net.*` builtins. Uses interior
/// mutability so `standard_lib::call` can stay `&RuntimeState`.
#[derive(Default)]
pub struct TcpRegistry {
    listeners: Mutex<HashMap<u64, TcpListener>>,
    streams: Mutex<HashMap<u64, TcpStream>>,
    next_listener_id: AtomicU64,
    next_stream_id: AtomicU64,
}

impl std::fmt::Debug for TcpRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TcpRegistry")
    }
}

impl TcpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_listener(&self, listener: TcpListener) -> CorvoResult<(u64, String)> {
        let local_addr = listener
            .local_addr()
            .map_err(|e| CorvoError::network(e.to_string()))?
            .to_string();
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        self.listeners.lock().unwrap().insert(id, listener);
        Ok((id, local_addr))
    }

    pub fn insert_stream(&self, stream: TcpStream) -> CorvoResult<(u64, String, String)> {
        let local_addr = stream
            .local_addr()
            .map_err(|e| CorvoError::network(e.to_string()))?
            .to_string();
        let peer_addr = stream
            .peer_addr()
            .map_err(|e| CorvoError::network(e.to_string()))?
            .to_string();
        let id = self.next_stream_id.fetch_add(1, Ordering::Relaxed);
        self.streams.lock().unwrap().insert(id, stream);
        Ok((id, local_addr, peer_addr))
    }

    pub fn accept(&self, listener_id: u64) -> CorvoResult<(u64, String, String)> {
        let (stream, _) = {
            let mut guard = self.listeners.lock().unwrap();
            let listener = guard.get_mut(&listener_id).ok_or_else(|| {
                CorvoError::invalid_argument("net.tcp_accept: unknown or closed tcp_listener")
            })?;
            listener
                .accept()
                .map_err(|e| CorvoError::network(e.to_string()))?
        };
        self.insert_stream(stream)
    }

    pub fn connect(addr: &str) -> CorvoResult<TcpStream> {
        let sock_addr: SocketAddr = addr
            .parse()
            .map_err(|_| CorvoError::invalid_argument("net.tcp_connect: invalid address"))?;
        TcpStream::connect(sock_addr).map_err(|e| CorvoError::network(e.to_string()))
    }

    pub fn read_stream(&self, stream_id: u64, max_bytes: usize) -> CorvoResult<String> {
        use std::io::Read;
        let mut guard = self.streams.lock().unwrap();
        let stream = guard.get_mut(&stream_id).ok_or_else(|| {
            CorvoError::invalid_argument("net.tcp_read: unknown or closed tcp_stream")
        })?;
        let mut buf = vec![0u8; max_bytes];
        let n = stream
            .read(&mut buf)
            .map_err(|e| CorvoError::network(e.to_string()))?;
        buf.truncate(n);
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    pub fn write_stream(&self, stream_id: u64, data: &[u8]) -> CorvoResult<()> {
        use std::io::Write;
        let mut guard = self.streams.lock().unwrap();
        let stream = guard.get_mut(&stream_id).ok_or_else(|| {
            CorvoError::invalid_argument("net.tcp_write: unknown or closed tcp_stream")
        })?;
        stream
            .write_all(data)
            .map_err(|e| CorvoError::network(e.to_string()))?;
        stream
            .flush()
            .map_err(|e| CorvoError::network(e.to_string()))?;
        Ok(())
    }

    pub fn remove_stream(&self, stream_id: u64) -> CorvoResult<()> {
        self.streams
            .lock()
            .unwrap()
            .remove(&stream_id)
            .ok_or_else(|| CorvoError::invalid_argument("net.tcp_close: unknown tcp_stream"))?;
        Ok(())
    }

    pub fn remove_listener(&self, listener_id: u64) -> CorvoResult<()> {
        self.listeners
            .lock()
            .unwrap()
            .remove(&listener_id)
            .ok_or_else(|| {
                CorvoError::invalid_argument("net.tcp_close_listener: unknown tcp_listener")
            })?;
        Ok(())
    }
}
