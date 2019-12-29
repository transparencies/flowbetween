use flo_stream::*;

use futures::stream::*;

///
/// Trait that can be implemented by items that represent a user interface
///
pub trait UserInterface<InputEvent, OutputUpdate, Error> {
    /// The type of the update stream for this UI
    type UpdateStream: Stream<Item = Result<OutputUpdate, Error>>;

    /// Retrieves an input event sink for this user interface
    fn get_input_sink(&self) -> Publisher<InputEvent>;

    /// Retrieves a view onto the update stream for this user interface
    fn get_updates(&self) -> Self::UpdateStream;
}
