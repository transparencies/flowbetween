///
/// Enumeration of events that can be generated by a Cocoa UI
///
#[derive(Clone, PartialEq, Debug)]
pub enum AppEvent {
    /// User has clicked on a view
    Click(usize, String)
}
