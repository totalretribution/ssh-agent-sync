use clap::Parser;
use colored::*;
use ssh_agent_sync::print_ssh_keys;
use ssh_agent_sync::get_ssh_keys;
use ssh_agent_sync::add_keys_to_config;
use ssh_agent_sync::constants;

#[derive(Parser)]
#[command(author, version, about = crate::constants::PROGRAM_NAME, arg_required_else_help = true)]
struct Args {
    /// Print keys like `ssh-add -L`
    #[arg(long)]
    print: bool,
    /// sync ssh agent keys to ssh config
    #[arg(long)]
    sync: bool, 
    // Force sync even if CRC matches (not implemented yet)
    #[arg(long)]
    force: bool,
}

fn main() {
    let args = Args::parse();
    let mut keys = get_ssh_keys().unwrap_or_default();

    println!(" {} {}", crate::constants::PROGRAM_NAME.bold().blue(), format!("v{}", crate::constants::PROGRAM_VERSION).dimmed());
    println!("{}", "───────────────────────".bright_black());

    if args.print {
        print_ssh_keys(&keys);
    }

    if args.sync {
        if let Err(e) = add_keys_to_config(&mut keys, args.force) {
            eprintln!("Failed to add keys to config: {}", e);
            std::process::exit(1);
        }
        println!("SSH keys synced to config successfully.");
    }
    
    std::process::exit(0);
}
