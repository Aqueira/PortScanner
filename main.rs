mod ftp;
mod http;
mod ports;
mod custom_errors;
mod ssh;
mod input;


use std::net::IpAddr;
use crate::input::{input, ParseInput};
use std::sync::Arc;
use tokio::time::timeout;
use tokio::net::TcpStream;
use custom_errors::Errors;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use ports::{Ports, ports};
use env_logger;
use log::{info, error, warn};
use crate::ftp::ftp_authorization;
use crate::http::get_version;
use crate::ssh::ssh_version;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Features{
    HttpVersion(String),
    FTPAuthorization(String),
    SSHVersion(String),
}
const TIME_OUT_PROGRAMS: u64 = 3;
const DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS: usize = 1000;

#[tokio::main]
async fn main() -> Result<(), Errors> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let parallel_tcp_connection_limiter = Arc::new(Semaphore::new(DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS));
    info!("Впишите чистое IP");
    let input_user = {
        let buffer = input()?;
        buffer.parse::<IpAddr>().map_err(|e|{
            error!("Ошибка парсинга - {}", e);
            Errors::Error
        })?;
        buffer
    };
    info!("От какого порта сканирование");
    let first_input_user: u16 = input()?.parse_input()?;
    info!("До какого порта сканирование");
    let second_input_user: u16 = input()?.parse_input()?;

    let buffer = input_user.clone();
    let async_thread: JoinHandle<Result<(), Errors>> = tokio::spawn(async move {
        scan_ports(buffer, first_input_user, second_input_user, parallel_tcp_connection_limiter).await?;
        Ok(())
    });
    async_thread.await??;

    let buffer = input_user.clone();
    let async_thread: JoinHandle<Result<(), Errors>> = tokio::spawn(async move {
        check_servers(buffer).await?;
        Ok(())
    });
    async_thread.await??;

    warn!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: String, port: u16){
    let port_type = ports(&port);
    let timeout_duration = Duration::from_secs(5);
    let target_cloned = target.clone();

    match timeout(timeout_duration, TcpStream::connect((target_cloned, port))).await{
        Ok(Ok(_)) => match port_type {
            Ports::HTTPS => info!("HTTPS - {}:{}", target, port),
            Ports::HTTP => info!("HTTP - {}:{}", target, port),
            Ports::FTP => info!("FTP - {}:{}", target, port),
            Ports::SSH => info!("SSH - {}:{}", target, port),
            Ports::SMTP => info!("SMTP - {}:{}", target, port),
            Ports::POP3 => info!("POP3 - {}:{}", target, port),
            Ports::IMAP =>  info!("IMAP - {}:{}", target, port),
            Ports::DNS => info!("DNS - {}:{}", target, port),
            Ports::MYSQL => info!("MYSQL - {}:{}", target, port),
            Ports::DHCP => info!("DHCP - {}:{}", target, port),
            Ports::RDP => info!("RDP - {}:{}", target, port),
            Ports::Telnet => info!("Telnet - {}:{}", target, port),
            Ports::Redis => info!("Redis - {}:{}", target, port),
            Ports::POP3S => info!("POP3S - {}:{}", target, port),
            Ports::LDAP => info!("LDAP - {}:{}", target, port),
            Ports::SNMP => info!("SNMP - {}:{}", target, port),
            Ports::SMTPS => info!("SMTPS - {}:{}", target, port),
            Ports::IMAPS => info!("IMAPS - {}:{}", target, port),
            Ports::PostgreSQL => info!("PostgreSQL - {}:{}", target, port),
            Ports::CPanel => info!("cPanel - {}:{}", target, port),
            Ports::HttpProxy => info!("HttpProxy - {}:{}", target, port),
            Ports::HostingHttp => info!("HostingHttp - {}:{}", target, port),
            Ports::WHM => info!("WHM - {}:{}", target, port),
            Ports::WhmSsL => info!("WHM_SSL - {}:{}", target, port),
            Ports::Other => info!("Other - {}:{}", target, port),
        },
        Ok(Err(e)) => warn!("Ошибка подключения к порту - {}", e),
        Err(_) => (),
    }
}

async fn scan_ports(target: String, start_port: u16, end_port: u16, parallel_tcp_connection_limiter: Arc<Semaphore>) -> Result<(), Errors>{
    let mut list = Vec::new();

    for port in start_port..=end_port{
        let cloned_parallel_tcp_connection_limiter = parallel_tcp_connection_limiter.clone();
        let target_clone = target.clone();
        let async_thread: JoinHandle<Result<(), Errors>> = tokio::spawn( async move  {
            let permit = cloned_parallel_tcp_connection_limiter.acquire().await.map_err(|e|{
                eprintln!("Ошибка получения разрешения симафора! - {}", e);
                Errors::Error
            })?;

            port_scan(target_clone, port).await;
            drop(permit);
            Ok(())
        });
        list.push(async_thread);
    }

    for handle in list{
        handle.await??;
    }

    Ok(())
}

async fn check_servers(target: String) -> Result<(), Errors>{
    loop{
        info!("1.Get => Http Headers");
        info!("2.Get => Ftp Authorization");
        info!("3.Get => SSH Version");
        info!("Other input => leave");
        let user_input: u8 = input()?.parse_input()?;
        match user_input{
            1 => {
                let features = get_version(&target).await.map_err(|_|{
                    error!("Ошибка получения версии!");
                    Errors::Error
                })?;
                for feature in &features{
                    match feature {
                        Features::HttpVersion(version) => info!("{}",version),
                        _ => (),
                    }
                }
            }
            2 => {
                let features = ftp_authorization(&target).await.map_err(|_|{
                    error!("Ошибка FTP Авторизации!");
                    Errors::Error
                })?;
                for feature in &features{
                    match feature{
                        Features::FTPAuthorization(auth) => info!("{}", auth),
                        _ => (),
                    }
                }
            }
            3 => {
                let features = ssh_version(&target).await.map_err(|_|{
                    error!("Ошибка получения версии SSH!");
                    Errors::Error
                })?;
                for feature in &features{
                    match feature{
                        Features::SSHVersion(version) => info!("{}", version),
                        _ => (),
                    }
                }
            }
            _ => break
        }
    }
    Ok(())
}
