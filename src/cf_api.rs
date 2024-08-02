use anyhow::Result;
use reqwest::blocking::Client;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CFResponse<T> {
    pub errors: Vec<CFMessage>,
    pub messages: Vec<CFMessage>,
    pub success: bool,
    pub result: Option<T>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CFMessage {
    pub code: u32,
    pub message: String,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct CFZone {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct CFDnsRecord {
    #[serde(rename = "type")]
    pub type_: CFRecordType,

    pub id: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum CFRecordType {
    A,
    AAAA,
}

pub fn get_zone_by_name(client: &Client, cf_token: &str, zone: &str) -> Result<CFZone> {
    let url = format!("https://api.cloudflare.com/client/v4/zones?name={}", zone);
    let resp = client
        .get(url)
        .bearer_auth(cf_token)
        .send()?
        .json::<CFResponse<Vec<CFZone>>>()?;
    if !resp.success {
        return Err(anyhow::anyhow!("Failed to get zone ID: {:?}", resp.errors));
    }
    let result = resp
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in response"))?;
    if result.len() != 1 {
        return Err(anyhow::anyhow!("Expected 1 zone, got {}", result.len()));
    }
    Ok(result[0].clone())
}

pub fn get_dns_record(
    client: &Client,
    cf_token: &str,
    zone_id: &str,
    domain: &str,
    record_type: CFRecordType,
) -> Result<CFDnsRecord> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type={}",
        zone_id,
        domain,
        match record_type {
            CFRecordType::A => "A",
            CFRecordType::AAAA => "AAAA",
        }
    );
    let resp = client
        .get(&url)
        .bearer_auth(cf_token)
        .send()?
        .json::<CFResponse<Vec<CFDnsRecord>>>()?;
    if !resp.success {
        return Err(anyhow::anyhow!(
            "Failed to get DNS record ID: {:?}",
            resp.errors
        ));
    }
    let result = resp
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in response"))?;
    if result.len() != 1 {
        return Err(anyhow::anyhow!("Expected 1 DNS record, got {:?}", result));
    }
    Ok(result[0].clone())
}

pub fn update_dns_record(
    client: &Client,
    cf_token: &str,
    zone_id: &str,
    record: &CFDnsRecord,
) -> Result<CFDnsRecord> {
    let url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
        zone_id, record.id
    );
    let resp = client
        .patch(&url)
        .bearer_auth(cf_token)
        .json(record)
        .send()?
        .json::<CFResponse<CFDnsRecord>>()?;
    if !resp.success {
        return Err(anyhow::anyhow!(
            "Failed to update DNS record: {:?}",
            resp.errors
        ));
    }
    Ok(resp.result.unwrap())
}
