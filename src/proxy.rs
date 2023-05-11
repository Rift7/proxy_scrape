use reqwest::Error as ReqwestError;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use anyhow::{Result, bail};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proxy {
    pub ip: String,
    pub port: u16,
    pub location: String,
    pub proxy_type: String,
    pub date: String,
}

pub async fn test_proxy(proxy: &Proxy) -> Result<(bool, String)> {
    let proxy_types = if proxy.proxy_type == "Unknown" {
        vec!["socks5", "https", "http"]
    } else {
        vec![&proxy.proxy_type[..]]
    };

    for proxy_type in proxy_types {
        let proxy_string = match proxy_type {
            "http" => format!("http://{}:{}", proxy.ip, proxy.port),
            "https" => format!("https://{}:{}", proxy.ip, proxy.port),
            "socks5" => format!("socks5://{}:{}", proxy.ip, proxy.port),
            _ => bail!("Unknown proxy type"),
        };

        let proxy_url = match reqwest::Url::parse(&proxy_string) {
            Ok(url) => url,
            Err(_) => {
                eprintln!("Failed to parse proxy URL: {}", proxy_string);
                continue;
            }
        };

        let client = match reqwest::Client::builder()
            .proxy(reqwest::Proxy::all(proxy_url)?)
            .build() {
            Ok(client) => client,
            Err(e) => {
                eprintln!("Failed to create client with proxy: {}. Error: {}", proxy_string, e);
                continue;
            }
        };

        let resp = match client.get("http://example.com").timeout(Duration::from_secs(2)).send().await {
            Ok(resp) => resp,
            Err(_) => continue,
        };

        // If the HTTP status is OK (200), the proxy is working
        if resp.status().is_success() {
            return Ok((true, proxy_type.to_string()));
        }
    }
    Ok((false, "None".to_string()))
}


pub async fn lookup_location(ip: &str) -> Result<String, ReqwestError> {
    // This function uses the `ip-api` service to look up the location of a given IP.
    // If the request is successful, it returns the country of the IP.
    let url = format!("http://ip-api.com/json/{}", ip);
    let resp = reqwest::get(&url).await?;
    let data: serde_json::Value = resp.json().await?;

    let country = data["country"].as_str().unwrap_or_default().to_string();

    Ok(country)
}
