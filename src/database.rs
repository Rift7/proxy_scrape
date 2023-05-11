use sled::Db;
use crate::proxy::Proxy;
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize)]
struct StoredProxy {
    ip: String,
    port: u16,
    location: String,
    proxy_type: String,
    date: String,
}

impl From<Proxy> for StoredProxy {
    fn from(proxy: Proxy) -> Self {
        StoredProxy {
            ip: proxy.ip,
            port: proxy.port,
            location: proxy.location,
            proxy_type: proxy.proxy_type,
            date: proxy.date,
        }
    }
}

impl From<StoredProxy> for Proxy {
    fn from(stored: StoredProxy) -> Self {
        Proxy {
            ip: stored.ip,
            port: stored.port,
            location: stored.location,
            proxy_type: stored.proxy_type,
            date: stored.date,
        }
    }
}

pub fn init_db(path: &str) -> Result<Db, sled::Error> {
    let db = sled::open(path)?;
    Ok(db)
}

pub fn insert_proxy(db: &Db, proxy: Proxy) -> Result<(), sled::Error> {
    let stored = StoredProxy::from(proxy);
    let serialized = bincode::serialize(&stored).unwrap();
    db.insert(&stored.ip, serialized)?;
    Ok(())
}

pub fn get_proxies(db: &Db) -> Result<Vec<Proxy>, Box<dyn std::error::Error>> {
    let mut proxies = Vec::new();

    for result in db.iter() {
        let (_k, v) = result?;
        let stored: StoredProxy = bincode::deserialize(&v)?;
        let proxy = Proxy {
            ip: stored.ip,
            port: stored.port,
            location: stored.location,
            proxy_type: stored.proxy_type,
            date: stored.date,
        };
        proxies.push(proxy);
    }

    Ok(proxies)
}
