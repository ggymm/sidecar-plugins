use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use clap::Parser;
use encoding_rs::GBK;
use once_cell::sync::Lazy;
use regex::Regex;
use sys_locale::get_locale;
use tokio::task::JoinSet;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{LookupIpStrategy, NameServerConfig, Protocol, ResolverConfig, ResolverOpts};

const TIMEOUT: Duration = Duration::from_secs(3);

const LANG_ZH: &str = "zh-CN";
const LANG_EN: &str = "en-US";

static LANG_LOCAL: Lazy<String> = Lazy::new(|| get_locale().unwrap_or_else(|| String::from(LANG_EN)));

static PING_PATTERNS: Lazy<HashMap<String, (Regex, Regex)>> = Lazy::new(|| {
    let mut patterns = HashMap::new();
    // English patterns
    patterns.insert(
        String::from(LANG_EN),
        (
            Regex::new(r"(\d+)% packet loss").unwrap(),
            Regex::new(r"Minimum = (\d+)ms, Maximum = (\d+)ms, Average = (\d+)ms").unwrap(),
        ),
    );
    // Chinese patterns
    patterns.insert(
        String::from(LANG_ZH),
        (
            Regex::new(r"丢失 = (\d+) \((\d+)% 丢失\)").unwrap(),
            Regex::new(r"最短 = (\d+)ms，最长 = (\d+)ms，平均 = (\d+)ms").unwrap(),
        ),
    );
    patterns
});

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Domain name to resolve
    #[arg(short, long, default_value = "auto.c3pool.org")]
    domain: String,

    /// Path to the nameservers file (one nameserver per line)
    #[arg(short, long, default_value = "nameservers.txt")]
    nameservers: PathBuf,
}

#[derive(Debug)]
struct PingMetrics {
    ip: IpAddr,
    packet_loss: u32,
    min_latency: u32,
    max_latency: u32,
    avg_latency: u32,
}

impl PingMetrics {
    fn new(ip: IpAddr) -> Self {
        Self {
            ip,
            packet_loss: 0,
            min_latency: 0,
            max_latency: 0,
            avg_latency: 0,
        }
    }
}

#[cfg(target_os = "windows")]
async fn ping_task(ip: IpAddr) -> io::Result<String> {
    let timeout_ms = TIMEOUT.as_millis().max(100) as u32;
    let output = Command::new("ping")
        .args(["/n", "4", "/w", &timeout_ms.to_string(), &ip.to_string()])
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to execute ping: {}", e)))?;

    if LANG_LOCAL.clone() == LANG_ZH {
        let (decoded, _, _) = GBK.decode(&output.stdout);
        Ok(decoded.into_owned())
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[cfg(not(target_os = "windows"))]
async fn ping_task(ip: IpAddr) -> io::Result<String> {
    let timeout_sec = TIMEOUT.as_secs().max(1);
    let output = Command::new("ping")
        .args(["-c", "4", "-W", &timeout_sec.to_string(), &ip.to_string()])
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to execute ping: {}", e)))?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn query_task(domain: &str, config: NameServerConfig) -> io::Result<Vec<IpAddr>> {
    let mut opts = ResolverOpts::default();
    opts.timeout = TIMEOUT;
    opts.attempts = 1;
    opts.validate = false;
    opts.use_hosts_file = false;
    opts.ip_strategy = LookupIpStrategy::Ipv4Only;
    let mut resolver = ResolverConfig::new();
    resolver.add_name_server(config);

    // 执行查询
    match TokioAsyncResolver::tokio(resolver, opts).lookup_ip(domain).await {
        Ok(response) => {
            let ips: Vec<_> = response.iter().collect();
            if !ips.is_empty() { Ok(ips) } else { Ok(vec![]) }
        }
        Err(_) => Ok(vec![]),
    }
}

async fn query_domain(domain: &str, nameservers: &[IpAddr]) -> io::Result<Vec<IpAddr>> {
    let mut rets = Vec::new();
    let mut tasks = JoinSet::new();
    for &nameserver in nameservers {
        let domain = domain.to_string();

        // 并发查询
        tasks.spawn(async move {
            println!(
                "Attempting to query domain '{}' using nameserver {} (UDP)",
                domain, nameserver
            );
            // 使用 UDP 协议查询
            let config = NameServerConfig {
                socket_addr: (nameserver, 53).into(),
                protocol: Protocol::Udp,
                tls_dns_name: None,
                trust_negative_responses: true,
                bind_addr: None,
            };
            let udp_result = query_task(&domain, config).await;
            match udp_result {
                Ok(ips) if !ips.is_empty() => Ok(ips),
                _ => {
                    println!(
                        "Attempting to query domain '{}' using nameserver {} (TCP)",
                        domain, nameserver
                    );
                    // 使用 TCP 协议查询
                    let config = NameServerConfig {
                        socket_addr: (nameserver, 53).into(),
                        protocol: Protocol::Tcp,
                        tls_dns_name: None,
                        trust_negative_responses: true,
                        bind_addr: None,
                    };
                    query_task(&domain, config).await
                }
            }
        });
    }

    // 收集结果
    while let Some(result) = tasks.join_next().await {
        if let Ok(Ok(ips)) = result {
            for ip in ips {
                if ip == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
                    continue;
                }
                if !rets.contains(&ip) {
                    rets.push(ip);
                }
            }
        }
    }

    Ok(rets)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // 读取 nameservers 文件
    let file = File::open(&cli.nameservers)?;
    let reader = BufReader::new(file);
    let nameservers: Vec<IpAddr> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| line.parse().ok())
        .collect();

    if nameservers.is_empty() {
        eprintln!("Error: nameservers file is empty or has incorrect format");
        std::process::exit(1);
    }

    // 根据 nameserver 查询 域名 对应的 ip 地址
    let all_ips = query_domain(&cli.domain, &nameservers).await?;
    if !all_ips.is_empty() {
        println!();
        println!("Query domain successful");

        // 执行 ping 命令检查连通性
        println!();
        println!("Check connection");
        let mut tasks = JoinSet::new();
        for ip in all_ips {
            tasks.spawn(async move { ping_task(ip).await.map(|output| (ip, output)) });
        }

        // 收集所有结果
        let mut results = Vec::new();
        while let Some(result) = tasks.join_next().await {
            match result {
                Ok(Ok((ip, output))) => {
                    println!("{}", output);

                    // 解析结果
                    let mut metrics = PingMetrics::new(ip);
                    let language = LANG_LOCAL.clone();
                    let patterns = PING_PATTERNS.get(&language).unwrap_or_else(|| {
                        eprintln!("Error: Unsupported language: {}", language);
                        panic!("Error: Unsupported language: {}", language)
                    });

                    if let Some(caps) = patterns.0.captures(&output) {
                        metrics.packet_loss = if language == LANG_ZH {
                            caps[2].parse().unwrap_or(100)
                        } else {
                            caps[1].parse().unwrap_or(100)
                        };
                    }
                    if let Some(caps) = patterns.1.captures(&output) {
                        metrics.min_latency = caps[1].parse().unwrap_or(0);
                        metrics.max_latency = caps[2].parse().unwrap_or(0);
                        metrics.avg_latency = caps[3].parse().unwrap_or(0);
                        results.push(metrics);
                    }
                }
                Ok(Err(err)) => eprintln!("Run ping command failed: {}", err),
                Err(err) => eprintln!("Run ping command failed: {}", err),
            }
        }

        if !results.is_empty() {
            println!("Display sorted results");
            results.sort_by(|a, b| {
                a.packet_loss
                    .cmp(&b.packet_loss)
                    .then(a.avg_latency.cmp(&b.avg_latency))
                    .then(a.max_latency.cmp(&b.max_latency))
            });

            // 打印排序后的 ip 列表
            println!();
            for metrics in results {
                println!(
                    "IP: {}, Packet Loss: {}%, Avg Latency: {}ms, Max Latency: {}ms",
                    metrics.ip, metrics.packet_loss, metrics.avg_latency, metrics.max_latency
                );
            }
        }
    } else {
        println!();
        println!("no ip addresses found");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_language() {
        let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        println!("Current locale: {}", locale);
    }
}
