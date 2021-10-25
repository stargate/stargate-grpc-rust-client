fn main() -> Result<(), Box<dyn std::error::Error>> {
    skeptic::generate_doc_tests(&["README.md"]);
    Ok(())
}
