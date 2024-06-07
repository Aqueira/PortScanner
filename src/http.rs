use crate::custom_errors::Error;
use crate::{Features, TIME_OUT_PROGRAMS};
use log::error;
use reqwest::{Client, Proxy, Response};
use std::net::IpAddr;
use std::time::Duration;

pub async fn http_features(target: &IpAddr, port_list: &Vec<u16>) -> Result<Vec<Features>, Error> {
    let mut http_features = vec![];

    let versions = get_version(target, port_list).await?;
    for version in versions {
        http_features.push(version);
    }

    Ok(http_features)
}

async fn get_version(target: &IpAddr, port_list: &Vec<u16>) -> Result<Vec<Features>, Error> {
    let client = create_client()?;
    let mut responses = vec![];

    for port in port_list {
        let url = format!("http://{}:{}", target, port);
        if let Some(response) = send_request(&url, &client).await {
            if let Some(version) = get_header(&response, target, port) {
                responses.push(Features::HttpVersion(version));
            } else {
                error!(
                    "Ошибка получения хэдера сервера! Статус - {}",
                    response.status().to_string()
                )
            }
        }
    }
    Ok(responses)
}

fn create_client() -> Result<Client, Error> {
    let proxy = create_proxy()?;
    Client::builder()
        .timeout(Duration::from_secs(TIME_OUT_PROGRAMS))
        .proxy(proxy)
        .build()
        .map_err(|e| Error::any("Ошибка создания клиента", e))
}

fn create_proxy() -> Result<Proxy, Error> {
    Proxy::https("116.203.207.197:8080").map_err(|e| Error::any("Ошибка создания прокси", e))
}

fn get_header(response: &Response, target: &IpAddr, port: &u16) -> Option<String> {
    if let Some(server_header) = response.headers().get("Server") {
        return Some(format!(
            "{}:{} -> version: {:?}",
            target, port, server_header
        ));
    }
    None
}

async fn send_request(url: &str, client: &Client) -> Option<Response> {
    return match client.get(url).send().await {
        Ok(response) => Some(response),
        Err(_) => None,
    };
}

