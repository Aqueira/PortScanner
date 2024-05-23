use crate::warn;
use std::time::Duration;
use log::{error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use crate::custom_errors::Errors;
use crate::{Features, TIME_OUT_PROGRAMS};

pub async fn ssh_version(target: &str) -> Result<Vec<Features>, Errors> {
    let mut features_list = vec![];
    let ssh_port = 22;
    let url = format!("{}:{}",target, &ssh_port);
    let timeout_duration = Duration::from_secs(TIME_OUT_PROGRAMS);

   if let Ok(Ok(mut tcp_stream)) = timeout (timeout_duration, TcpStream::connect(&url)).await{
       let request = b"SSH-2.0\n";
       tcp_stream.write_all(request).await.map_err(|e|{
           error!("Ошибка отправки! - {}", e);
           Errors::Error
       })?;

       let mut buffer = [0; 1024];
       tcp_stream.read(&mut buffer).await.map_err(|e|{
           error!("Ошибка записи! - {}", e);
           Errors::Error
       })?;

       if buffer.is_empty(){
           error!("Пустой буффер!");
           return Err(Errors::Error)
       };
       let version_brute = String::from_utf8_lossy(&mut buffer);
       if let Some(version) = version_brute.lines().next(){
           features_list.push(Features::SSHVersion(version.to_string()));
       }
   }
   else{
       warn!("Таймаут - ошибка подключения к порту!");
   }
    Ok(features_list)
}
