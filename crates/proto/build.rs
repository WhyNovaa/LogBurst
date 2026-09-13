fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/log.proto");

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&["proto/log.proto"], &["proto"])?;

    Ok(())
}
