use std::time::Duration;
use log::error;
use reqwest::{Client, Proxy};
use crate::custom_errors::Errors;
use crate::ports::Ports;
use crate::TIME_OUT_PROGRAMS;

pub enum HttpFeature{
    ServerVersion(String)
}
pub trait GetServerVersion{
    async fn get_version(target: &str) -> Result<Vec<HttpFeature>, Errors>;
}
impl GetServerVersion for Ports{
    async fn get_version(target: &str) -> Result<Vec<HttpFeature>, Errors> {
        let port_list = [80, 443, 8080, 8443, 8880];
        let client = create_client()?;
        let mut features_list = vec![];

        for port in &port_list {
            let url = format!("http://{}:{}", target, port);
            match client.get(&url).send().await {
                Ok(response) => {
                    if let Some(server_header) = response.headers().get("Server") {
                        let version = format!("{}:{} -> {:?}",target, port, server_header);
                        features_list.push(HttpFeature::ServerVersion(version));
                    }
                    else {
                        error!("Ошибка получения хэдера сервера! Статус - {}", response.status().to_string())
                    }
                }
                Err(_) => (),
            };
        }
        Ok(features_list)
    }
}
fn create_client() -> Result<Client, Errors>{
    let proxy = Proxy::https("116.203.207.197:8080").map_err(|e|{
        error!("Ошибка создания прокси клиента! - {}", e);
        Errors::Error
    })?;

    Client::builder()
        .timeout(Duration::from_secs(TIME_OUT_PROGRAMS))
        .proxy(proxy)
        .build()
        .map_err(|e|{
            error!("Ошибка создания клиента! - {}", e);
            Errors::Error
        })
}
