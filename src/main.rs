use crate::ftp::ftp_features;
use crate::http::http_features;
use crate::ssh::ssh_features;
use aqueiralibrary::{input, ParseInput};
use custom_errors::Error;
use env_logger;
use log::{info, warn};
use ports::Port;
use std::any::Any;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio::time::Duration;

mod custom_errors;
mod ftp;
mod http;
mod ports;
mod ssh;

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
    info!("Впишите IPv4");
    let input_user: IpAddr = input()?.parsing()?;
    info!("От какого порта сканирование");
    let first_input_user: u16 = input()?.parsing()?;
    info!("До какого порта сканирование");
    let second_input_user: u16 = input()?.parsing()?;

    let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        scan_ports(
            input_user,
            first_input_user,
            second_input_user,
            parallel_tcp_connection_limiter,
        )
        .await
    });
    async_thread.await??;

    let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        user_interface(
            get_features(&input_user)
                .await
                .map_err(|e| Error::any("Ошибка получения особенностей!", e))?,
        )
        .await
    });
    async_thread.await??;

    warn!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: IpAddr, port: Port) {
    let timeout_duration = Duration::from_secs(5);

    match timeout(timeout_duration, TcpStream::connect((target, port.num))).await {
        Ok(Ok(_)) => info!("{} - {}:{}", port.name, target, port.num),
        Ok(Err(e)) => warn!("Ошибка подключения к порту - {}", e),
        Err(_) => (),
    }
}

async fn scan_ports(
    target: IpAddr,
    start_port: u16,
    end_port: u16,
    parallel_tcp_connection_limiter: Arc<Semaphore>,
) -> Result<(), Error> {
    let mut list = Vec::new();

    for port in (start_port..=end_port).map(|port| Port::from(port)) {
        let cloned_parallel_tcp_connection_limiter = parallel_tcp_connection_limiter.clone();
        let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let permit = cloned_parallel_tcp_connection_limiter
                .acquire()
                .await
                .map_err(|e| {
                    Error::any(
                        "Достигнут максимум параллельных подключений сканирования",
                        e,
                    )
                })?;
            port_scan(target, port).await;
            drop(permit);
            Ok(())
        });
        list.push(async_thread);
    }

    for handle in list {
        handle.await??;
    }

    Ok(())
}

async fn user_interface(features: Vec<Features>) -> Result<(), Error> {
    for feature in &features {
        match feature {
            Features::FTPAuth(auth) => warn!("{}", auth),
            Features::HttpVersion(version) => warn!("{}", version),
            Features::SSHVersion(version) => warn!("{}", version),
            _ => (),
        }
    }
    Ok(())
}

async fn get_features(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let http_features = http_features(target)
        .await
        .map_err(|e| Error::any("Ошибка получения HTTP особенностей!", e))?;
    let ssh_features = ssh_features(target)
        .await
        .map_err(|e| Error::any("Ошибка получения SSH особенностей!", e))?;
    let ftp_features = ftp_features(target)
        .await
        .map_err(|e| Error::any("Ошибка получения FTP особенностей!", e))?;

    let new_vector: Vec<Features> = vec![http_features, ssh_features, ftp_features]
        .into_iter()
        .flatten()
        .collect();

    Ok(new_vector)
}
