#[no_mangle]
pub fn call(input: &str) -> Result<String, anyhow::Error> {
    Ok(format!("test-lib-b-{input}"))
}
