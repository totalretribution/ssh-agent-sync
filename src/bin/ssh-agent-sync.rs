use clap::Parser;
use ssh_agent_sync::print_ssh_keys;
use ssh_agent_sync::get_ssh_keys;
use ssh_agent_sync::add_keys_to_config;

#[derive(Parser)]
#[command(author, version, about = "SSH Agent sync CLI", arg_required_else_help = true)]
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

    if args.print {
        print_ssh_keys(&keys);
    }
    if args.sync {
        if let Err(e) = add_keys_to_config(&mut keys, args.force) {
            eprintln!("Failed to add keys to config: {}", e);
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}
