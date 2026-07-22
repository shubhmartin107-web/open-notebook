fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("proto");

    let out_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    prost_build::Config::new()
        .out_dir(&out_dir)
        .compile_protos(
            &[proto_dir.join("onb.proto")],
            &[&proto_dir],
        )?;

    Ok(())
}
