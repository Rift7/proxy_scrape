use reqwest::Error as ReqwestError;
use scraper::{Html, Selector, ElementRef};
use crate::proxy::Proxy;
use regex::Regex;
use std::str::FromStr;

pub async fn get_html(url: &str) -> Result<String, ReqwestError> {
    let resp = reqwest::get(url).await?;
    let body = resp.text().await?;
    Ok(body)
}

pub fn scrape_proxies(html: String) -> Vec<Proxy> {
    let document = Html::parse_document(&html);
    let table_selector = Selector::parse("tbody tr").unwrap();
    
    let mut proxies: Vec<Proxy> = document
        .select(&table_selector)
        .filter_map(parse_proxy)
        .collect();

    if proxies.is_empty() {
        let re = Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}):(\d{1,5})\b").unwrap();
        for cap in re.captures_iter(&html) {
            if let Ok(ip) = std::net::IpAddr::from_str(&cap[1]) {
                match cap[2].parse::<u16>() {
                    Ok(port) => {
                        let proxy = Proxy {
                            ip: ip.to_string(),
                            port,
                            location: "Unknown".to_string(),
                            proxy_type: "Unknown".to_string(),
                            date: "Unknown".to_string(),
                        };
                        proxies.push(proxy);
                    },
                    Err(_) => {
                        // eprintln!("Failed to parse port number from scraped proxy: {}", &cap[2]);
                    }
                }
            }
        }
    }

    proxies
}


fn parse_proxy(row: ElementRef) -> Option<Proxy> {
    let data: Vec<_> = row.text().collect();
    if data.len() >= 2 {
        if let Ok(ip) = std::net::IpAddr::from_str(&data[0]) {
            if let Ok(port) = data[1].parse::<u16>() {
                return Some(Proxy {
                    ip: ip.to_string(),
                    port,
                    // If we can't find certain data, we can just use some default values.
                    location: "Unknown".to_string(),
                    proxy_type: "Unknown".to_string(),
                    date: "Unknown".to_string(),
                });
            } else {
                // eprintln!("Failed to parse port number from scraped proxy: {}", &data[1]);
            }
        } else {
            // eprintln!("Failed to parse IP address from scraped proxy: {}", &data[0]);
        }
    }
    None
}
