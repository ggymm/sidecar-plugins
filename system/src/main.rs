mod basic_info;
mod dynamic_info;

use serde_json::json;

fn print_basic_info() {
    let output = json!({
        "basic":  basic_info::collect_basic_info(),
        "timestamp": chrono::Local::now().to_rfc3339(),
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn print_dynamic_info() {
    let output = json!({
        "dynamic":  dynamic_info::collect_dynamic_info(),
        "timestamp": chrono::Local::now().to_rfc3339(),
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <basic|dynamic> [interval_seconds]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "basic" => print_basic_info(),
        "dynamic" => {
            if args.len() >= 3 {
                match args[2].parse::<u64>() {
                    Ok(interval) => loop {
                        print_dynamic_info();
                        std::thread::sleep(std::time::Duration::from_secs(interval));
                    },
                    Err(_) => {
                        eprintln!("Error: Invalid interval seconds. Must be a positive number.");
                        std::process::exit(1);
                    }
                }
            } else {
                print_dynamic_info();
            }
        }
        _ => {
            eprintln!("Error: Invalid command. Use 'basic' or 'dynamic'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_info() {
        print_basic_info();
    }

    #[test]
    fn test_dynamic_info() {
        print_dynamic_info();
    }
}
