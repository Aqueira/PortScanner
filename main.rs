mod custom_errors;
use std::sync::Arc;
use tokio::time::timeout;
use tokio::net::TcpStream;
use custom_errors::Errors;
use std::io;
use std::net::IpAddr;
use reqwest::{Client, Proxy};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::Duration;



#[tokio::main]
async fn main() -> Result<(), Errors>{
    let semaphore = Arc::new(Semaphore::new(1000));

    println!("Впишите чистое IP");
    let input_user = ip_input_user()?;
    println!("От какого порта сканирование");
    let first_input_user = int_input_user()?;
    println!("До какого порта сканирование");
    let second_input_user = int_input_user()?;

    let async_thread: JoinHandle<Result<(), Errors>> =  tokio::spawn(async move{
        scan_ports(&input_user, first_input_user, second_input_user, semaphore).await?;
        Ok(())
    });
    async_thread.await??;

    println!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: &str, port: u16) {
    let port_type = ports(&port);
    let timeout_duration = Duration::from_secs(5);
    match timeout(timeout_duration, TcpStream::connect((target, port))).await{
        Ok(Ok(_)) => match port_type {
            Ports::HTTPS => println!("HTTPS - {}:{}", target, port),
            Ports::HTTP => println!("HTTP - {}:{}", target, port),
            Ports::FTP => println!("FTP - {}:{}", target, port),
            Ports::SSH => println!("SSH - {}:{}", target, port),
            Ports::SMTP => println!("SMTP - {}:{}", target, port),
            Ports::POP3 => println!("POP3 - {}:{}", target, port),
            Ports::IMAP => println!("IMAP - {}:{}", target, port),
            Ports::DNS => println!("DNS - {}:{}", target, port),
            Ports::MYSQL => println!("MYSQL - {}:{}", target, port),
            Ports::DHCP => println!("DHCP - {}:{}", target, port),
            Ports::RDP => println!("RDP - {}:{}", target, port),
            Ports::Telnet => println!("Telnet - {}:{}", target, port),
            Ports::Redis => println!("Redis - {}:{}", target, port),
            Ports::POP3S => println!("POP3S - {}:{}", target, port),
            Ports::LDAP => println!("LDAP - {}:{}", target, port),
            Ports::SNMP => println!("SNMP - {}:{}", target, port),
            Ports::SMTPS => println!("SMTPS - {}:{}", target, port),
            Ports::IMAPS => println!("IMAPS - {}:{}", target, port),
            Ports::PostgreSQL => println!("PostgreSQL - {}:{}", target, port),
            Ports::CPanel => println!("cPanel - {}:{}", target, port),
            Ports::HttpProxy => println!("HttpProxy - {}:{}", target, port),
            Ports::HostingHttp => println!("HostingHttp - {}:{}", target, port),
            Ports::WHM => println!("WHM - {}:{}", target, port),
            Ports::WhmSsL => println!("WHM_SSL - {}:{}", target, port),
            Ports::Other => println!("Other - {}:{}", target, port),

        },
        Ok(Err(e)) => println!("Ошибка подключения к порту - {}", e),
        Err(_) => (),
    }
}

async fn scan_ports(target: &str, start_port: u16, end_port: u16, semaphore: Arc<Semaphore>) -> Result<(), Errors>{
    let mut list = Vec::new();

    for port in start_port..=end_port{
        let buffer = target.to_string();
        let cloned_semaphore = semaphore.clone();
        let handle: JoinHandle<Result<(), Errors>> = tokio::spawn( async move  {
            let permit = cloned_semaphore.acquire().await.map_err(|e|{
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
        eprintln!("Ошибка пользовательствого ввода");
        Errors::Error
    })?;
    buffer.trim().parse::<IpAddr>().map_err(|e|{
        eprintln!("Введен неправильный Ip-Address - {}", e);
        Errors::Error
    })?;

    Ok(buffer)
}

fn int_input_user() -> Result<u16, Errors>{
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).map_err(|_| {
        eprintln!("Ошибка пользовательствого ввода");
        Errors::Error
    })?;
    buffer.trim().parse::<u16>().map_err(|e| {
        eprintln!("Ошибка парсинга - {}", e);
        Errors::Error
    })
}

async fn get_version(target: &str) -> Result<(), Errors>{
    let port_list = [80, 443, 8080, 8443, 8880];
    let proxy = create_proxy()?;
    let client = create_client(proxy)?;

    for port in &port_list{
        let url = format!("http://{}:{}", target, port);
        match client.get(&url).send().await{
            Ok(response) => {
                if let Some(server_header) = response.headers().get("Server"){
                    let version = server_header.to_str().unwrap();
                    println!("-----------------------------------------------");
                    println!("Версия сервера порта: {} - {}", port, version);
                    println!("-----------------------------------------------");
                    println!(" ");
                }
                else{
                    eprintln!("Ошибка получения хэдера сервера! Статус - {}", response.status().to_string())
                }
            }
            Err(e) => {
                eprintln!("Ошибка получения ответа - {}", e)
            },
        };
    }

    Ok(())
}

async fn check_auth_ftp(target: &str) -> Result<(), Errors> {
    let url = format!("{}:21", target);
    let timeout_dur = Duration::from_secs(5);

    let mut stream = match timeout(timeout_dur, TcpStream::connect(&url)).await {
        Ok(Ok(stream)) => stream,
        Err(e) => {
            eprintln!("Таймаут истек: {}", e);
            return Err(Errors::Error);
        },
        Ok(Err(e)) => {
            eprintln!("Неопознанная ошибка - {}", e);
            return Err(Errors::Error);
        }
    };

    let request = b"USER anonymous\r\n";
    stream.write_all(request).await.map_err(|e|{
        eprintln!("Ошибка записи данных в поток! - {}", e);
        Errors::Error
    })?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.map_err(|e|{
        eprintln!("Ошибка записи данных из потока! - {}", e);
        Errors::Error
    })?;

    let response = &buffer[..n];
    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

fn ports(port: &u16) -> Ports{
    match port{
        20 => Ports::FTP,
        21 => Ports::FTP,
        22 => Ports::SSH,
        23 => Ports::Telnet,
        25 => Ports::SMTP,
        53 => Ports::DNS,
        67 => Ports::DHCP,
        68 => Ports::DHCP,
        110 => Ports::POP3,
        143 => Ports::IMAP,
        161 => Ports::SNMP,
        162 => Ports::SNMP,
        389 => Ports::LDAP,
        443 => Ports::HTTPS,
        465 => Ports::SMTPS,
        587 => Ports::SMTP,
        993 => Ports::IMAPS,
        995 => Ports::POP3S,
        2083 => Ports::CPanel,
        2086 => Ports::WHM,
        2087 => Ports::WhmSsL,
        3306 => Ports::MYSQL,
        3389 => Ports::RDP,
        5432 => Ports::PostgreSQL,
        6379 => Ports::Redis,
        8080 => Ports::HttpProxy,
        8880 => Ports::HTTP,
        8443 => Ports::HostingHttp,
        80 => Ports::HTTP,
        _ => Ports::Other,
    }
}

fn create_client(proxy: Proxy) -> Result<Client, Errors>{
    Client::builder()
        .timeout(Duration::from_secs(5))
        .proxy(proxy)
        .build()
        .map_err(|e|{
            eprintln!("Ошибка создания клиента! - {}", e);
            Errors::Error
        })
}

fn create_proxy() -> Result<Proxy, Errors>{
    Proxy::https("116.203.207.197:8080").map_err(|e|{
        eprintln!("Ошибка создания прокси! - {}", e);
        Errors::Error
    })
}

fn input_user() -> Result<String, Errors>{
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).map_err(|e| {
        eprintln!("Ошибка ввода пользователя - {}", e);
        Errors::Error
    })?;
    Ok(buffer.trim().to_string())
}

async fn check_servers(target: &str) -> Result<(), Errors>{
    println!("Хотите получить версию сервера?  Y/N");

    if input_user()?.to_lowercase() == "y"{
        println!("Выберите: \n1.Http-Https\n2.FTP");
        loop{
            let user_input = int_input_user()?;
            match user_input{
                1 => {
                    get_version(target).await?;
                    println!("Выберите: \n1.HTTP-HTTPS\n2.FTP");
                },
                2 => {
                    if let Err(_) = check_auth_ftp(target).await{}
                    println!("Выберите: \n1.Http-Https\n2.FTP");
                }
                _ => break
            }
        }
    }
    Ok(())
}

enum Ports{
    FTP,
    SSH,
    Telnet,
    SMTP,
    POP3,
    IMAP,
    HTTPS,
    HTTP,
    DNS,
    DHCP,
    SNMP,
    LDAP,
    SMTPS,
    IMAPS,
    POP3S,
    MYSQL,
    RDP,
    PostgreSQL,
    Redis,
    Other,
    CPanel,
    HttpProxy,
    HostingHttp,
    WHM,
    WhmSsL,
}
