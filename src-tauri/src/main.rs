fn main() {
    if let Err(error) = pebble_lib::run() {
        eprintln!("Pebble failed to start: {error}");
        std::process::exit(1);
    }
}
