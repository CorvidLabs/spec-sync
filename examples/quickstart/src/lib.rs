//! A trivial greeter. Used as the canonical SpecSync 5-minute example.

/// Returns a greeting for `name`.
///
/// # Examples
///
/// ```
/// use greeter::greet;
/// assert_eq!(greet("world"), "hello, world!");
/// ```
pub fn greet(name: &str) -> String {
    format!("hello, {name}!")
}
