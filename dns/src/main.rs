use std::fs::File;
use std::io::{self, BufRead};
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use tokio::task::JoinSet;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Domain name to resolve
    #[arg(short, long)]
    domain: String,

    /// Path to the nameservers file (one nameserver per line)
    #[arg(short, long)]
    nameservers: PathBuf,
}

async fn query_task(config: NameServerConfig, domain: &str) -> io::Result<Vec<IpAddr>> {
    let protocol = if config.protocol == Protocol::Tcp { "TCP" } else { "UDP" };
    let nameserver = config.socket_addr.ip();
    println!("attempting to query domain '{}' using nameserver {} ({})", domain, nameserver, protocol);

    // 构造配置
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_secs(3);
    opts.attempts = 1;
    opts.validate = false;
    opts.use_hosts_file = false;

    // 执行查询
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::from_parts(None, vec![], vec![config]),
        opts,
    );

    match resolver.lookup_ip(domain).await {
        Ok(response) => {
            let ips: Vec<_> = response.iter().collect();
            if !ips.is_empty() {
                println!("query successful for {} ({}) - found {} ip addresses:", nameserver, protocol, ips.len());
                Ok(ips)
            } else {
                println!("no ip addresses found for {} ({})", nameserver, protocol);
                Ok(vec![])
            }
        }
        Err(err) => {
            eprintln!("query failed for {} ({}): {}", nameserver, protocol, err);
            Ok(vec![])
        }
    }
}

async fn query_domain(domain: &str, nameservers: &[IpAddr]) -> io::Result<Vec<IpAddr>> {
    let mut rets = Vec::new();
    let mut tasks = JoinSet::new();
    for &nameserver in nameservers {
        let domain = domain.to_string();

        // 并发查询
        tasks.spawn(async move {
            // 使用 UDP 协议查询
            let config = NameServerConfig {
                socket_addr: (nameserver, 53).into(),
                protocol: Protocol::Udp, // 初始配置为UDP
                tls_dns_name: None,
                trust_negative_responses: true,
                bind_addr: None,
            };
            let udp_result = query_task(config, &domain).await;
            match udp_result {
                Ok(ips) if !ips.is_empty() => Ok(ips),
                _ => {
                    // 使用 TCP 协议查询
                    let config = NameServerConfig {
                        socket_addr: (nameserver, 53).into(),
                        protocol: Protocol::Tcp, // 切换为TCP
                        tls_dns_name: None,
                        trust_negative_responses: true,
                        bind_addr: None,
                    };
                    query_task(config, &domain).await
                }
            }
        });
    }

    // 收集结果
    while let Some(result) = tasks.join_next().await {
        if let Ok(Ok(ips)) = result {
            // 添加新的IP（避免重复）
            for ip in ips {
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
    let reader = io::BufReader::new(file);
    let nameservers: Vec<IpAddr> = reader.lines().filter_map(|line| line.ok()).filter_map(|line| line.parse().ok()).collect();

    if nameservers.is_empty() {
        eprintln!("error: nameservers file is empty or has incorrect format");
        std::process::exit(1);
    }

    // 执行DNS查询
    let all_ips = query_domain(&cli.domain, &nameservers).await?;

    // 显示所有收集到的IP地址
    if !all_ips.is_empty() {
        println!();
        println!("all resolved ip addresses:");
        for ip in all_ips {
            println!("  {}", ip);
        }
    } else {
        println!();
        println!("no ip addresses found");
    }

    Ok(())
}
