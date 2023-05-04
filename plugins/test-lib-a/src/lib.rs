#[no_mangle]
pub fn call(_input: ()) -> Result<String, anyhow::Error> {
    Ok("test-lib-a".into())
}
