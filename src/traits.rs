use crate::TIME_OUT_PROGRAMS;
use log::warn;
use std::borrow::Cow;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

pub struct ProtocolOperation;

pub trait ProtocolOperations {
    fn get_converted_response_to_utf8(buffer: &[u8], read: usize) -> Cow<str> {
        String::from_utf8_lossy(&buffer[..read])
    }

    async fn read_request(buffer: &mut [u8; 1024], stream: &mut TcpStream) -> Option<usize> {
        return match stream.read(buffer).await {
            Ok(read) => Some(read),
            Err(e) => {
                warn!("Ошибка записи значения в буффер! -> {}", e);
                None
            }
        };
    }

    async fn get_tcp_connection_stream(url: &str, port: &u16) -> Option<TcpStream> {
        let timeout_duration = Duration::from_secs(TIME_OUT_PROGRAMS);
        match timeout(timeout_duration, TcpStream::connect(url)).await {
            Ok(Ok(stream)) => Some(stream),
            Ok(Err(e)) => {
                warn!("Ошибка подключения к порту - {}: {}", port, e);
                None
            }
            Err(e) => {
                warn!("├─ Таймаут подключения к порту! - {}: {}", port, e);
                None
            }
        }
    }

    async fn write_request(request: &[u8], stream: &mut TcpStream) {
        match stream.write_all(request).await {
            Ok(_) => (),
            Err(e) => warn!("Ошибка отправки запроса! - {}", e),
        }
    }
}

impl ProtocolOperations for ProtocolOperation {}
