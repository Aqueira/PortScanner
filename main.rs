mod custom_errors;
use tokio::time::timeout;
use tokio::net::TcpStream;
use custom_errors::Errors;
use std::io;
use std::net::IpAddr;
use reqwest::{Client, Proxy};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;


#[tokio::main]
async fn main() -> Result<(), Errors>{
    println!("Впишите чистое IP");
    let input_user = ip_input_user()?;
    println!("От какого порта сканирование");
    let first_input_user = int_input_user()?;
    println!("До какого порта сканирование");
    let second_input_user = int_input_user()?;
    scan_ports(&input_user, first_input_user, second_input_user).await?;
    println!("Выполнение программы закончено!");
    Ok(())
}

async fn port_scan(target: &str, port: u16) {
    let port_type = ports(&port);
    let timeout_duration = Duration::from_secs(8);
    match timeout(timeout_duration, TcpStream::connect((target, port))).await{
        Ok(Ok(_)) => match port_type {
            Ports::HTTPS => println!("Connected port: https://{}:{}", target, port),
            Ports::HTTP => println!("Connected port: http://{}:{}", target, port),
            Ports::FTP => println!("Connected port: ftp://{}:{}", target, port),
            Ports::SSH => println!("Connected port: SSH -> {}:{}", target, port),
            Ports::SMTP => println!("Connected port: smtp://{}:{}", target, port),
            Ports::POP3 => println!("Connected port: pop3://{}:{}", target, port),
            Ports::IMAP => println!("Connected port: imap://{}:{}", target, port),
            Ports::Other => println!("Connected port: {}:{}", target, port),
        },
        Ok(Err(e)) => println!("Ошибка подключения к порту - {}", e),
        Err(_) => (),
    }
}

async fn scan_ports(target: &str, start_port: u16, end_port: u16) -> Result<(), Errors>{
    let mut list = Vec::new();

    for port in start_port..=end_port{
        let buffer = target.to_string();
        let handle = tokio::spawn( async move  {
            port_scan(&buffer, port).await;
        });
        list.push(handle);
    }

    for handle in list{
        handle.await?;
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

    Ok(buffer.trim().to_string())
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
    let port_list = [80,443,8080];
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
                    println!("Ошибка получения хэдера сервера! Статус - {}", response.status().to_string())
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
        eprintln!("Ошибка отправки запроса! - {}", e);
        Errors::Error
    })?;

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await.map_err(|e|{
        eprintln!("Ошибка записи авторизации - {}", e);
        Errors::Error
    })?;

    let response = &buffer[..n];
    println!("Response: {}", String::from_utf8_lossy(response));

    Ok(())
}

fn ports(port: &u16) -> Ports{
    let port_type = match port{
        21 => Ports::FTP,
        22 => Ports::SSH,
        25 => Ports::SMTP,
        110 => Ports::POP3,
        143 => Ports::IMAP,
        443 => Ports::HTTPS,
        80 => Ports::HTTP,
        _ => Ports::Other,
    };
    port_type
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
        println!("Выберите: \n1.Http-Https\n2.Проверка - FTP\n");
        loop{
            let user_input = int_input_user()?;
            match user_input{
                1 => {
                    get_version(target).await?;
                    println!("Повторный выбор: ");
                },
                2 => {
                    if let Err(_) = check_auth_ftp(target).await{}
                    println!("Повторный выбор: ");
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
    SMTP,
    POP3,
    IMAP,
    HTTPS,
    HTTP,
    Other
}
