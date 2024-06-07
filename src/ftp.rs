use crate::traits::{ProtocolOperation, ProtocolOperations};
use crate::Error;
use crate::Features;
use crate::Features::FTPAuth;
use std::net::IpAddr;
use tokio::net::TcpStream;

pub async fn ftp_features(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let mut ftp_features = vec![];

    let auths = ftp_authorization(target).await?;
    for auth in auths {
        ftp_features.push(auth);
    }
    Ok(ftp_features)
}

async fn ftp_authorization(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let ftp_ports = vec![20, 21];
    let mut features = vec![];

    for port in &ftp_ports {
        let url = format!("{}:{}", target, port);
        if let Some(mut stream) = establish_connection(&url, port).await {
            send_request(&mut stream).await;
            let mut buffer = [0u8; 1024];
            if let Some(read) = get_response_request(&mut stream, &mut buffer).await {
                let text = convert_text_to_string(&mut buffer, read);
                let result = get_status_auth(&text, target, port);
                features.push(result);
            }
        }
    }
    Ok(features)
}

fn get_status_auth(buffered_text: &str, target: &IpAddr, port: &u16) -> Features {
    let target_line = "No anonymous login";

    return if buffered_text.contains(target_line) {
        FTPAuth(format!("Authorization Rejected -> {}:{}", target, port))
    } else {
        FTPAuth(format!("Authorization Accepted -> {}:{}", target, port))
    };
}

async fn send_request(stream: &mut TcpStream) {
    let request = b"USER anonymous\r\n";
    ProtocolOperation::write_request(request, stream).await;
}

async fn get_response_request(stream: &mut TcpStream, buffer: &mut [u8; 1024]) -> Option<usize> {
    if let Some(read) = ProtocolOperation::read_request(buffer, stream).await {
        return Some(read);
    }
    None
}

async fn establish_connection(endpoint: &str, port: &u16) -> Option<TcpStream> {
    ProtocolOperation::get_tcp_connection_stream(endpoint, port).await
}

fn convert_text_to_string(not_converted_text: &[u8], read: usize) -> String {
    let text = ProtocolOperation::get_converted_response_to_utf8(not_converted_text, read);
    text.to_string()
}

