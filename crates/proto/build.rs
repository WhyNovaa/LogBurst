fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/log.proto");
    println!("cargo:rerun-if-changed=proto/kafka_log.proto");

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/log.proto", "proto/kafka_log.proto"], &["proto"])?;

    Ok(())
}
