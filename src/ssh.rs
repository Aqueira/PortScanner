use crate::traits::{ProtocolOperation, ProtocolOperations};
use crate::warn;
use crate::Error;
use crate::Features;
use std::net::IpAddr;
use tokio::net::TcpStream;

const SSH_PORT: u16 = 22;
const NAME_SSH_REQUEST: &[u8; 8] = b"SSH-2.0\n";

pub async fn ssh_features(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let mut ssh_features = vec![];

    if let Some(ssh_version) = ssh_version(target).await {
        ssh_features.push(ssh_version);
    }

    Ok(ssh_features)
}

async fn ssh_version(target: &IpAddr) -> Option<Features> {
    let end_point = format!("{}:{}", target, &SSH_PORT);

    if let Some(mut stream) = establish_connection(&end_point).await {
        send_ssh_request(&mut stream).await;
        if let Some(features) = read_ssh_response(&mut stream).await {
            return Some(features);
        };
        return None;
    }
    None
}

async fn establish_connection(endpoint: &str) -> Option<TcpStream> {
    ProtocolOperation::get_tcp_connection_stream(endpoint, &SSH_PORT).await
}

async fn send_ssh_request(stream: &mut TcpStream) {
    ProtocolOperation::write_request(NAME_SSH_REQUEST, stream).await
}

async fn read_ssh_response(stream: &mut TcpStream) -> Option<Features> {
    let mut buffer = [0; 1024];
    if let Some(read) = ProtocolOperation::read_request(&mut buffer, stream).await {
        if read == 0 {
            warn!("Буффер пуст!");
            return None;
        };

        let response_text = ProtocolOperation::get_converted_response_to_utf8(&buffer, read);
        response_text
            .lines()
            .next()
            .map(|version| Features::SSHVersion(version.to_string()))
    } else {
        return None;
    }
}
