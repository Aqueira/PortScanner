use crate::ftp::ftp_features;
use crate::http::http_features;
use crate::ssh::ssh_features;
use custom_errors::Error;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Features{
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

    let parallel_tcp_connection_limiter = Arc::new(Semaphore::new(DEFAULT_MAX_PARALLEL_TCP_CONNECTIONS));
    info!("Впишите чистое IP");
    let input_user: IpAddr = input()?.parsing()?;
    info!("От какого порта сканирование");
    let first_input_user: u16 = input()?.parsing()?;
    info!("До какого порта сканирование");
    let second_input_user: u16 = input()?.parsing()?;

    let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        scan_ports(input_user, first_input_user, second_input_user, parallel_tcp_connection_limiter).await?;
        Ok(())
    });
    async_thread.await??;

    let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        user_interface(get_features(&input_user).await.map_err(|e| Error::any("Ошибка получения особенностей!", e))?).await?;
        Ok(())
    });
    async_thread.await??;

    warn!("Выполнение программы закончено!");
    Ok(())

}

async fn port_scan(target: IpAddr, port: u16){
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

async fn scan_ports(target: IpAddr, start_port: u16, end_port: u16, parallel_tcp_connection_limiter: Arc<Semaphore>) -> Result<(), Error>{
    let mut list = Vec::new();

    for port in start_port..=end_port{
        let cloned_parallel_tcp_connection_limiter = parallel_tcp_connection_limiter.clone();
        let async_thread: JoinHandle<Result<(), Error>> = tokio::spawn( async move  {
            let permit = cloned_parallel_tcp_connection_limiter.acquire().await.map_err(|e| Error::any("Ошибка получения разрешения симафора!", e))?;
            port_scan(target, port).await;
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

async fn user_interface(features: Vec<Features>) -> Result<(), Error>{
    for feature in &features{
        match feature{
            Features::FTPAuth(auth) => warn!("{}", auth),
            Features::HttpVersion(version) => warn!("{}", version),
            Features::SSHVersion(version) => warn!("{}", version),
            _ => (),
        }
    }
    Ok(())
}

async fn get_features(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let http_features = http_features(target).await.map_err(|e| Error::any("Ошибка получения HTTP особенностей!", e))?;
    let ssh_features = ssh_features(target).await.map_err(|e| Error::any("Ошибка получения SSH особенностей!", e))?;
    let ftp_features = ftp_features(target).await.map_err(|e| Error::any("Ошибка получения FTP особенностей!", e))?;

    let new_vector: Vec<Features> = vec![http_features, ssh_features, ftp_features].into_iter().flatten().collect();

    Ok(new_vector)
}
