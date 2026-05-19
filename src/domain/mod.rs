pub mod arrangement;
pub mod category;
pub mod service;
pub mod song;

pub(crate) fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
