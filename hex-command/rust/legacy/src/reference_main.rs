fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = if args.is_empty() {
        hex_command_core::reference::run()
    } else {
        hex_command_core::simulation::cli::run(&args).map_err(Into::into)
    };
    hex_command_core::ai::profiling::write();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
