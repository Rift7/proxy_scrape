/*!
Main entry point for ProxyScraper

This delightful little program will scrape your chosen websites for proxy information
faster than a raccoon in a dumpster. It carefully extracts and tests proxies, storing
the good ones in a SLED database, ready for all your clandestine operations.

And if that wasn't enough, it can export these proxies to a nifty CSV file for your
viewing pleasure.
*/

use crate::database::{init_db, insert_proxy, get_proxies};
use crate::scraper::{get_html, scrape_proxies};
use crate::proxy::{test_proxy, lookup_location};
use crate::exporter::export_to_csv;
use std::env;
use tokio;
use colored::*;
use structopt::StructOpt;

mod database;
mod scraper;
mod proxy;
mod exporter;

#[derive(Debug, StructOpt)]
#[structopt(name = "proxy-scraper", about = "Scrapes and tests proxies.")]

struct Opt {
    /// Activate verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Write output to csv
    #[structopt(long = "to-csv", parse(from_os_str))]
    to_csv: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    
    // URLs of websites to scrape
    let urls = vec![
        "https://free-proxy-list.net/anonymous-proxy.html",
        "https://free-proxy-list.net/"
        // ...
    ];

    let current_dir = env::current_dir()?;
    // Path to the SLED database
    let db_path = current_dir.join("proxy_db");

    // Initialize the database (or panic if it can't be done)
    let db = init_db(db_path.to_str().unwrap()).expect("Failed to open database. Maybe it went to the wrong port?");

    // Handle the --to-csv argument
    if let Some(csv_path) = opt.to_csv {
        // Fetch the proxies from the database and export them to a CSV file
        if opt.verbose {
            println!("Fetching proxies from database...");
        }
        match get_proxies(&db) {
            Ok(proxies) => {
                if opt.verbose {
                    println!("Exporting {} proxies to CSV...", proxies.len());
                }
                if let Err(e) = export_to_csv(&proxies, &csv_path) {
                    eprintln!("Failed to export proxies to CSV: {}", e.to_string().red());
                }
            }
            Err(e) => eprintln!("Failed to fetch proxies from database: {}", e.to_string().red()),
        }
    } else {

        for url in &urls {
            if opt.verbose {
                println!("Scraping proxies from {}...", url);
            }
        
            // Fetch the HTML and scrape proxies from it
            match get_html(url).await {
                Ok(html) => {
                    let proxies = scrape_proxies(html);
                    if opt.verbose {
                        println!("Found {} proxies.", proxies.len().to_string().bright_green());
                    }
        
                    for mut proxy in proxies {
                        // Test the proxy. If it's active, store it in the database.
                        match test_proxy(&mut proxy).await {
                            Ok((true, proxy_type)) => {
                                // Set the proxy type
                                proxy.proxy_type = proxy_type;
                        
                                // Lookup the location of the proxy only if it's "Unknown"
                                if proxy.location == "Unknown" {
                                    match lookup_location(&proxy.ip).await {
                                        Ok(location) => {
                                            proxy.location = location;
                                        }
                                        Err(e) => eprintln!("Failed to lookup proxy location: {}", e.to_string().red()),
                                    }
                                }
                                if opt.verbose {
                                    println!("Proxy {}:{} is {} with type: {} location: {}", proxy.ip.cyan().bold(), proxy.port.to_string().bright_blue().bold(), "active".green().bold(), proxy.proxy_type.green().bold(), proxy.location.green().bold());
                                }
                                
                                // Insert the proxy into the database
                                if let Err(e) = insert_proxy(&db, proxy) {
                                    eprintln!("Failed to insert proxy into database: {}", e.to_string().red());
                                }
                            }
                            Ok((false, _)) => {
                                if opt.verbose {
                                    println!("Proxy {}:{} is {}. We'll check back later.", proxy.ip.cyan().bold(), proxy.port.to_string().bright_blue().bold(), "offline".red().bold());
                                }
                            }
                            Err(e) => eprintln!("Failed to test proxy: {}", e.to_string().red()),
                        }                    
                    }
                }
                Err(e) => eprintln!("Failed to get HTML from {}: {}", url, e.to_string().red()),
            }
        }

        if opt.verbose {
            println!("All done! Now go and surf the web like a ninja!");
        }
    }

    Ok(())
}