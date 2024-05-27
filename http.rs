use std::net::IpAddr;
use std::time::Duration;
use log::error;
use reqwest::{Client, Proxy};
use crate::{Features, TIME_OUT_PROGRAMS};
use crate::Error;

pub async fn http_features(target: &IpAddr) -> Result<Vec<Features>, Error>{
    let mut http_features = vec![];

    let versions = get_version(target).await.map_err(|e| Error::any("Ошибка получения версии http", e))?;
    for version in versions{
        http_features.push(version);
    }

    Ok(http_features)
}
async fn get_version(target: &IpAddr) -> Result<Vec<Features>, Error> {
    let port_list = [80, 443, 8080, 8443, 8880];
    let client = create_client()?;
    let mut responses = vec![];

    for port in &port_list {
        let url = format!("http://{}:{}", target, port);
        match client.get(&url).send().await {
            Ok(response) => {
                if let Some(server_header) = response.headers().get("Server") {
                    let version = format!("{}:{} -> {:?}",target, port, server_header);
                    responses.push(Features::HttpVersion(version));
                }
                else {
                    error!("Ошибка получения хэдера сервера! Статус - {}", response.status().to_string())
                }
            }
            Err(_) => (),
        };
    }
    Ok(responses)
}

fn create_client() -> Result<Client, Error>{
    let proxy = Proxy::https("116.203.207.197:8080").map_err(|e| Error::any("Ошибка создания прокси!", e))?;

    Client::builder()
        .timeout(Duration::from_secs(TIME_OUT_PROGRAMS))
        .proxy(proxy)
        .build()
        .map_err(|e| Error::any("Ошибка сорздания клиента!", e))
}
