fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("api/stargate.proto")?;
    skeptic::generate_doc_tests(&["README.md"]);
    Ok(())
}
