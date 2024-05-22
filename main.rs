mod custom_errors;
mod ftp;
mod http;
mod ports;
mod ssh;

use crate::ssh::{SSHVersion, SSHFeatures};
use crate::http::{GetServerVersion, HttpFeature};
use crate::ftp::{FTPFeature, FTPAuthorization};
use std::sync::Arc;
use tokio::time::timeout;
use tokio::net::TcpStream;
use custom_errors::Errors;
use std::io;
use std::net::IpAddr;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use ports::{Ports, ports};
use env_logger;
use log::{info, error, warn};
const DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS: usize = 1000;
const TIME_OUT_PROGRAMS: u64 = 3;

#[tokio::main]
async fn main() -> Result<(), Errors> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let parallel_tcp_connection_limiter = Arc::new(Semaphore::new(DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS));
    info!("Впишите чистое IP");
    let input_user = ip_input_user()?;
    info!("От какого порта сканирование");
    let first_input_user = int_input_user()?;
    info!("До какого порта сканирование");
    let second_input_user = int_input_user()?;

    let async_thread: JoinHandle<Result<(), Errors>> =  tokio::spawn(async move{
        scan_ports(&input_user, first_input_user, second_input_user, parallel_tcp_connection_limiter).await?;
        Ok(())
    });
    async_thread.await??;

    warn!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: &str, port: u16) {
    let port_type = ports(&port);
    let timeout_duration = Duration::from_secs(5);
    match timeout(timeout_duration, TcpStream::connect((target, port))).await{
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

async fn scan_ports(target: &str, start_port: u16, end_port: u16, parallel_tcp_connection_limiter: Arc<Semaphore>) -> Result<(), Errors>{
    let mut list = Vec::new();

    for port in start_port..=end_port{
        let buffer = target.to_string();
        let cloned_parallel_tcp_connection_limiter = parallel_tcp_connection_limiter.clone();

        let handle: JoinHandle<Result<(), Errors>> = tokio::spawn( async move  {
            let permit = cloned_parallel_tcp_connection_limiter.acquire().await.map_err(|e|{
                eprintln!("Ошибка получения разрешения симафора! - {}", e);
                Errors::Error
            })?;

            port_scan(&buffer, port).await;
            drop(permit);
            Ok(())
        });
        list.push(handle);
    }

    for handle in list{
        handle.await??;
    }

    check_servers(target).await?;
    Ok(())
}

fn ip_input_user() -> Result<String, Errors>{
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).map_err(|_| {
        error!("Ошибка пользовательствого ввода");
        Errors::Error
    })?;

    buffer.trim().parse::<IpAddr>().map_err(|e|{
        error!("Введен неправильный Ip-Address - {}", e);
        Errors::Error
    })?;

    Ok(buffer.trim().to_string())
}

fn int_input_user() -> Result<u16, Errors>{
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).map_err(|_| {
        error!("Ошибка пользовательствого ввода");
        Errors::Error
    })?;

    buffer.trim().parse::<u16>().map_err(|e| {
        error!("Ошибка парсинга - {}", e);
        Errors::Error
    })
}

async fn check_servers(target: &str) -> Result<(), Errors>{
    loop{
        info!("1.Get => Http Headers");
        info!("2.Get => Ftp Authorization");
        info!("3.Get => SSH Version");
        info!("Other input => leave");
        let user_input = int_input_user()? as u8;
        match user_input{
            1 => {
                let features = Ports::get_version(target).await.map_err(|e|{
                    error!("Ошибка получения версии! - {}", e);
                    Errors::Error
                })?;
                for feature in &features{
                    match feature {
                        HttpFeature::ServerVersion(version) => info!("{}",version),
                    }
                }
            }
            2 => {
                let features = Ports::ftp_authorization(target).await.map_err(|e|{
                    error!("Ошибка FTP Авторизации! - {}", e);
                    Errors::Error
                })?;
                for feature in &features{
                    match feature{
                        FTPFeature::Anon(auth) => info!("{}", auth),
                    }
                }
            }
            3 => {
                let features = Ports::ssh_version(target).await.map_err(|e|{
                    error!("Ошибка получения версии SSH! - {}", e);
                    Errors::Error
                })?;
                for feature in &features{
                    match feature{
                        SSHFeatures::SSHVersion(version) => info!("{}", version),
                    }
                }
            }
            _ => break
        }
    }
    Ok(())
}
