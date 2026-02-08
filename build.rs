fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = tonic_build::configure();

    config.compile_protos(
        &["src/interfaces/grpc/proto/log.proto"],
        &["src/interfaces/grpc/proto"],
    )?;

    Ok(())
}
