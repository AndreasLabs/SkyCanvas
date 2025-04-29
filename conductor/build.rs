fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/groundlink.proto")?;
    tonic_build::compile_protos("../proto/types.proto")?;
    Ok(())
}