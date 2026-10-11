/// A pack size as typed: `Ok(None)` when left blank, `Ok(Some)` once it is a whole
/// number above zero.
pub fn parse_purchase_size(value: &str) -> Result<Option<i32>, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    match value.parse::<i32>() {
        Ok(size) if size > 0 => Ok(Some(size)),
        Ok(_) => Err("Must be more than 0".to_string()),
        Err(_) => Err("Must be a whole number".to_string()),
    }
}
