/// Retire obsolete audited-reopened intent without accepting its old implementation.
pub fn retire_change(
    _root: &Path, _id: &str, _actor: String, _reason: String,
    _replacement: String, _authority_source: String,
) -> Result<ChangeRecord, String> {
    Err("Not implemented".into())
}

/// Validate retained evidence custody for a terminal nonacceptance disposition.
pub fn validate_retirement(_root: &Path, _record: &ChangeRecord) -> Result<(), String> {
    Err("Not implemented".into())
}
