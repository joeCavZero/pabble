use pabble::cli::PabbleCLI;

fn main() {
    let cli = PabbleCLI::new(std::env::args().collect());
    cli.run();
}
