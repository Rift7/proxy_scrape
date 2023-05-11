use csv::Writer;
use std::error::Error;
use std::path::Path;
use crate::proxy::Proxy;

pub fn export_to_csv(proxies: &[Proxy], path: &Path) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(path)?;

    // Write the headers
    wtr.write_record(&["IP", "Port", "Location", "Type", "Date"])?;

    // Write each record
    for proxy in proxies {
        wtr.write_record(&[&proxy.ip, &proxy.port.to_string(), &proxy.location, &proxy.proxy_type, &proxy.date])?;
    }

    // Ensure everything is written to disk
    wtr.flush()?;

    Ok(())
}
