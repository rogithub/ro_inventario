use std::fmt::Display;

#[askama::filter_fn]
pub fn pesos<T: Display>(value: T, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(format!("{:.2}", value))
}
