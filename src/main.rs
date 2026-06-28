use pebble::cli::PebbleCLI;

fn main() {
    let cli = PebbleCLI::new(std::env::args().collect());
    cli.run();
}
