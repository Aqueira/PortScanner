mod custom_errors;
mod ftp;
mod http;
mod ports;
mod ssh;

use crate::ftp::ftp_features;
use crate::http::http_features;
use crate::ssh::ssh_features;
use aqueiralibrary::{input, ParseInput};
use custom_errors::Error;
use env_logger;
use log::{info, warn};
use ports::Port;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio::time::Duration;

#[derive(PartialEq, Debug)]
pub enum Features {
    //#HTTP
    HttpVersion(String),
    //...
    //#FTP
    FTPAuth(String),
    //...
    //#SSH
    SSHVersion(String),
    //...
    //Пустая хуетень
    Empty(),
}
const TIME_OUT_PROGRAMS: u64 = 3;
const DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let parallel_tcp_connection_limiter =
        Arc::new(Semaphore::new(DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS));
    info!("Впишите чистое IP");
    let input_user: IpAddr = input()?.parsing()?;
    info!("От какого порта сканирование");
    let first_input_user: u16 = input()?.parsing()?;
    info!("До какого порта сканирование");
    let second_input_user: u16 = input()?.parsing()?;

    let async_thread: JoinHandle<Result<Vec<u16>, Error>> = tokio::spawn(async move {
        let ports = scan_ports(
            input_user,
            first_input_user,
            second_input_user,
            parallel_tcp_connection_limiter,
        )
        .await?;
        Ok(ports)
    });
    let ports = async_thread.await??;

    let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        user_interface(get_features(&input_user, &ports).await?).await?;
        Ok(())
    });
    async_thread.await??;

    warn!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: IpAddr, port: Port) -> Option<u16> {
    let timeout_duration = Duration::from_secs(TIME_OUT_PROGRAMS);

    match timeout(timeout_duration, TcpStream::connect((target, port.num))).await {
        Ok(Ok(_)) => {
            info!("{} -> {}:{}", port.name, target, port.num);
            return Some(port.num);
        }
        Ok(Err(e)) => {
            warn!("Ошибка подключения к порту - {}", e);
            None
        }
        Err(_) => None,
    }
}

async fn scan_ports(
    target: IpAddr,
    start_port: u16,
    end_port: u16,
    parallel_tcp_connection_limiter: Arc<Semaphore>,
) -> Result<Vec<u16>, Error> {
    let mut async_tasks = Vec::new();
    let vector_ports = Arc::new(Mutex::new(vec![]));

    for port in (start_port..=end_port).map(|port| Port::from(port)) {
        let cloned_parallel_tcp_connection_limiter = parallel_tcp_connection_limiter.clone();
        let cloned_vector_ports = vector_ports.clone();

        let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let permit = cloned_parallel_tcp_connection_limiter
                .acquire()
                .await
                .map_err(|e| Error::any("Достигнут лимит разрешений симафора!", e))?;

            if let Some(scanned_port) = port_scan(target, port).await {
                let mut mutex_guard = cloned_vector_ports.lock().await;
                mutex_guard.push(scanned_port);
                drop(mutex_guard)
            }

            drop(permit);
            Ok(())
        });
        async_tasks.push(async_thread);
    }

    for handle in async_tasks {
        handle.await??;
    }

    let vector_u16 = vector_ports.lock().await.clone();
    Ok(vector_u16)
}

async fn user_interface(features: Vec<Features>) -> Result<(), Error> {
    for feature in &features {
        match feature {
            Features::FTPAuth(auth) => warn!("├─ [FTP] Аутентификация: {}", auth),
            Features::HttpVersion(version) => warn!("├─ [HTTP] Версия: {}", version),
            Features::SSHVersion(version) => warn!("├─ [SSH] Версия: {}", version),
            _ => (),
        }
    }
    Ok(())
}

async fn get_features(target: &IpAddr, port_list: &Vec<u16>) -> Result<Vec<Features>, Error> {
    let http_features = http_features(target, port_list).await?;
    let ssh_features = ssh_features(target).await?;
    let ftp_features = ftp_features(target).await?;

    let new_vector: Vec<Features> = vec![http_features, ssh_features, ftp_features]
        .into_iter()
        .flatten()
        .collect();

    Ok(new_vector)
}
