mod dynamic_info;
mod static_info;

use std::env;

fn print_static_info() {
    let info = static_info::collect_static_info();
    let json = serde_json::to_string_pretty(&info).unwrap();
    println!("{}", json);
}

fn print_dynamic_info() {
    println!("Dynamic info not implemented yet");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <static|dynamic>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "static" => print_static_info(),
        "dynamic" => print_dynamic_info(),
        _ => {
            eprintln!("Error: Invalid command. Use 'static' or 'dynamic'");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_system_info() {
        print_static_info();
    }
}
