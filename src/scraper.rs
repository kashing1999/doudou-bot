use super::utils;
use chrono::offset::Local;
use regex::Regex;

use html_parser::Dom;

#[derive(Debug)]
pub struct Product {
    pub vendor: String,
    pub url: String,
}

impl Product {
    pub fn new(vendor: String, url: String) -> Product {
        Product { vendor, url }
    }
}

pub struct Website {
    vendor: String,
    url: String,
    script: String,
    search_term: String,
}

impl Website {
    pub fn new(vendor: String, url: String, script: String, search_term: String) -> Website {
        Website {
            vendor,
            url,
            script,
            search_term,
        }
    }

    pub async fn get_products(&self) -> Option<Vec<String>> {
        let client = reqwest::Client::new();

        let resp = client
            .get(&self.url)
            .header(reqwest::header::USER_AGENT, super::USER_AGENT)
            .send()
            .await;

        match resp {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::OK {
                    let html = resp.text().await.ok()?;
                    let json = Dom::parse(&html).unwrap().to_json_pretty().ok()?;
                    let output = utils::execute_script(&self.script, &json).await;
                    let re = Regex::new(&self.search_term).unwrap();
                    let products = output
                        .split_whitespace()
                        .filter(|x| re.is_match(x))
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    Some(products)
                } else {
                    let current_time = Local::now().to_string();
                    println!(
                        "{}: {} got status code {}",
                        current_time,
                        resp.url().as_str(),
                        resp.status().as_u16()
                    );
                    None
                }
            }
            Err(e) => {
                println!(
                    "Error while making web request to {}: {:?}",
                    self.get_url(),
                    e
                );
                None
            }
        }
    }

    pub async fn query_website(&self) -> Result<Vec<Product>, String> {
        let mut result = vec![];
        if let Some(products) = self.get_products().await {
            match products.len() {
                0 => {
                    return Err(format!(
                        "@here Hmm, not getting any products from {}: {}",
                        self.get_vendor(),
                        self.get_url()
                    ))
                }
                _ => {
                    for p in products {
                        let product = Product::new(self.get_vendor().to_string(), p.clone());
                        result.push(product);
                    }
                }
            };
        };
        Ok(result)
    }

    pub fn get_vendor(&self) -> &str {
        &self.vendor
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
}
