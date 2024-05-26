use std::net::IpAddr;
use crate::{Features, TIME_OUT_PROGRAMS};
use std::time::Duration;
use log::{warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use crate::custom_errors::Errors;


pub async fn ftp_authorization(target: &IpAddr) -> Result<Vec<Features>, Errors>{
    let timeout_tcp_duration = Duration::from_secs(TIME_OUT_PROGRAMS);
    let ftp_ports =  vec![20,21];
    let mut features = vec![];

    for port in &ftp_ports{
        let url = format!("{}:{}", target, port);
        let mut stream = match timeout(timeout_tcp_duration, TcpStream::connect(&url)).await{
            Ok(Ok(stream)) => stream,
            Err(e) => {
                warn!("Таймаут подключения к порту - {} не установлено! - {}", port, e);
                continue
            },
            Ok(Err(e)) => {
                warn!("Ошибка подключения к порту - {} не установлено! - {}", port,  e);
                continue
            },
        };

        let request = b"USER anonymous\r\n";
        match stream.write_all(request).await{
            Ok(_) => (),
            Err(e) => warn!("Ошибка отправки запроса! - {}", e)
        }

        let mut buffer = [0; 1024];
        match stream.read(&mut buffer).await{
            Ok(read) => {
                let response = &buffer[0..read];
                let buffered_text = String::from_utf8_lossy(response);
                let target_line = "No anonymous login";

                if buffered_text.contains(target_line){
                    features.push(Features::FTPAuthorization(format!("FTP connect => Rejected - {}:{}",target, port)));
                }
                else{
                    features.push(Features::FTPAuthorization(format!("FTP connect => Accepted - {}:{}",target, port)));
                }
            }
            Err(e) => warn!("Ошибка чтения! - {}", e)
        }
    }
    Ok(features)
}
