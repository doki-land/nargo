use clap::Parser;
use nargo_lsp::run_stdio;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// 启用调试模式
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.debug {
        println!("Nargo LSP server starting in debug mode...");
    }

    if let Err(e) = run_stdio().await {
        eprintln!("Error running LSP server: {}", e);
        std::process::exit(1);
    }
}
