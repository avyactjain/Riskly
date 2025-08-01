fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile_protos(
            &["riskly-protos/riskly.proto"],
            &["riskly-protos"],
        )?;
    Ok(())
}
