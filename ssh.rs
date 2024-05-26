use std::net::IpAddr;
use crate::warn;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use crate::{Features, TIME_OUT_PROGRAMS};
use crate::custom_errors::Errors;


pub async fn ssh_features(target: &IpAddr) -> Result<Vec<Features>, Errors>{
    let mut ssh_features = vec![];

    ssh_features.push(ssh_version(target).await.map_err(|e|{
        Errors::error("Ошибка получения SSH версии!", e);
    })?);


    Ok(ssh_features)
}
async fn ssh_version(target: &IpAddr) -> Result<Features, Errors> {
    let ssh_port = 22;
    let url = format!("{}:{}",target, &ssh_port);
    let timeout_duration = Duration::from_secs(TIME_OUT_PROGRAMS);

   match timeout (timeout_duration, TcpStream::connect(&url)).await{
       Ok(Ok(mut tcp_stream)) => {
           let request = b"SSH-2.0\n";
           match tcp_stream.write_all(request).await{
               Ok(_) => {
                   let mut buffer = [0; 1024];
                   match tcp_stream.read(&mut buffer).await{
                       Ok(read) => {
                           if read == 0{
                               warn!("Пустой буфер!");
                           }
                           let version_brute = String::from_utf8_lossy(&buffer[..read]);
                           if let Some(version) = version_brute.lines().next(){
                              return Ok(Features::SSHVersion(version.to_string()))
                           }
                       }
                       Err(e) => warn!("Ошибка чтения - {}", e)
                   }
               }
               Err(e) => warn!("Ошибка отправки! - {}", e)
           }
       }
       Ok(Err(e)) => warn!("Ошибка подключения к порту! - {}", e),
       Err(e) => warn!("Таймаут подключения к порту! - {}", e),
   };
    Ok(Features::Empty())
}

