use anyhow::Result;
use cf_api::{update_dns_record, CFDnsRecord};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

mod cf_api;

const INADDR_ANY_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const INADDR_ANY_V6: IpAddr = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0));

enum IPProto {
    V4,
    V6,
}

#[derive(Parser, Debug)]
struct Args {
    // Cloudflare access token
    #[arg(long)]
    cf_token: String,

    // zone name
    #[arg(long)]
    zone: String,

    // domain name in the zone
    #[arg(long)]
    domain: String,

    // Don't actually update IP
    #[arg(long)]
    dry_run: bool,
}

fn get_my_ip(proto: IPProto) -> Result<String> {
    let client = reqwest::blocking::ClientBuilder::new()
        .local_address(match proto {
            IPProto::V4 => INADDR_ANY_V4,
            IPProto::V6 => INADDR_ANY_V6,
        })
        .build()?;
    let body = client
        .get("https://cloudflare.com/cdn-cgi/trace")
        .send()?
        .text()?;
    let ip = body
        .lines()
        .find(|line| line.starts_with("ip="))
        .ok_or_else(|| anyhow::anyhow!("No IP in response"))?;
    Ok(ip[3..].to_string())
}

fn main() {
    let args = Args::parse();
    let ip4 = get_my_ip(IPProto::V4).unwrap();

    println!("My IPv4 address is: {}", ip4);

    let client = reqwest::blocking::Client::new();
    let zone = cf_api::get_zone_by_name(&client, &args.cf_token, &args.zone).unwrap();
    println!("Zone ID: {}", zone.id);

    let record = cf_api::get_dns_record(
        &client,
        &args.cf_token,
        &zone.id,
        &args.domain,
        cf_api::CFRecordType::A,
    )
    .unwrap();
    println!("Existing DNS record: {:?}", record);

    if record.content == ip4 {
        println!("IP is up to date, nothing to do");
        return;
    }

    let new_record = CFDnsRecord{
        content: ip4,
        ..record.clone()
    };

    if args.dry_run {
        println!("Dry run, not updating record.\n  OLD: {:?}\n  NEW: {:?}", record, new_record);
        return;
    } else {
        let updated_record = update_dns_record(&client, &args.cf_token, &zone.id, &new_record).unwrap();
        println!("Updated record to {:?}", updated_record);
    }
}
