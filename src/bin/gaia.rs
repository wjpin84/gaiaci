fn main() -> Result<(), Box<dyn std::error::Error>> {
    gaiaci::logging::init_logging();
    gaiaci::cli::run()
}