
use crate::{Ports, TIME_OUT_PROGRAMS};
use std::time::Duration;
use log::{error, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use crate::custom_errors::Errors;
use tokio::time::timeout;
#[derive(PartialEq)]
#[derive(Debug)]
pub enum FTPFeature {
    Anon(String)
}
pub trait FTPAuthorization{
    async fn ftp_authorization(target: &str) -> Result<Vec<FTPFeature>, Errors>;
}
impl FTPAuthorization for Ports {
    async fn ftp_authorization(target: &str) -> Result<Vec<FTPFeature>, Errors>{
        let timeout_tcp_duration = Duration::from_secs(TIME_OUT_PROGRAMS);
        let ftp_ports =  vec![20,21];
        let mut features = vec![];

        for port in &ftp_ports{
            let url = format!("{}:{}", target, port);
            let mut stream = match timeout(timeout_tcp_duration, TcpStream::connect(&url)).await{
                Ok(Ok(stream)) => stream,
                Err(e) => {
                    warn!("Подключение до порта {} не установлено! - {}", port, e);
                    continue
                },
                Ok(Err(_)) => continue,
            };

            let request = b"USER anonymous\r\n";
            stream.write_all(request).await.map_err(|e|{
                error!("Ошибка записи данных в поток! - {}", e);
                Errors::Error
            })?;

            let mut buffer = [0; 1024];
            let read = stream.read(&mut buffer).await.map_err(|e|{
                error!("Ошибка чтения данных из потока! - {}", e);
                Errors::Error
            })?;

            let response = &buffer[0..read];
            let buffered_text = String::from_utf8_lossy(response);
            let target_line = "No anonymous login";

            if buffered_text.contains(target_line){
                features.push(FTPFeature::Anon(format!("FTP connect => Rejected - {}:{}",target, port)));
            }
            else{
                features.push(FTPFeature::Anon(format!("FTP connect => Accepted - {}:{}",target, port)));
            }
        }
        Ok(features)
    }
}
