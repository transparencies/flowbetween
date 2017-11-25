use super::frame::*;
use super::attributes::*;
use super::frame_parameter::*;

///
/// A layer represents a renderable plane in an animation
///
pub trait Layer : HasAttributes {
    ///
    /// Retrieves a frame from this layer with the specified parameters
    ///
    fn get_frame(&self, Box<Iterator<Item = FrameParameter>>) -> Box<Frame>;
}
