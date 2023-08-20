fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/inner.proto")?;
    tonic_build::compile_protos("proto/outer.proto")?;
    Ok(())
}
