fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = tonic_build::configure();

    config.compile_protos(&["proto/log.proto"], &["proto"])?;

    Ok(())
}
