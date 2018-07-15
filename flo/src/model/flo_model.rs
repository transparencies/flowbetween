use super::tools::*;
use super::frame::*;
use super::timeline::*;
use super::selection::*;

use flo_binding::*;
use flo_animation::*;
use futures::*;

use std::ops::{Deref, Range};
use std::time::Duration;
use std::sync::*;

///
/// The model for the animation editor
/// 
pub struct FloModel<Anim: Animation> {
    /// The animation that is being edited
    animation: Arc<Anim>,

    /// The status of the currently selected tool
    tools: ToolModel<Anim>,

    /// The timeline model
    timeline: TimelineModel<Anim>,

    /// The frame model
    frame: FrameModel,

    /// The selection model
    selection: SelectionModel,

    /// The size of the animation
    pub size: BindRef<(f64, f64)>,

    /// The underlying size binding
    size_binding: Binding<(f64, f64)>,

    /// Counter used to set an edit ID for the frame (essentially indicates when the frame has been redrawn)
    frame_edit_counter: Binding<u64>
}

impl<Anim: Animation+'static> FloModel<Anim> {
    ///
    /// Creates a new model
    /// 
    pub fn new(animation: Anim) -> FloModel<Anim> {
        let animation           = Arc::new(animation);
        let tools               = ToolModel::new();
        let timeline            = TimelineModel::new(Arc::clone(&animation));
        let frame_edit_counter  = bind(0);
        let frame               = FrameModel::new(Arc::clone(&animation), BindRef::new(&timeline.current_time), BindRef::new(&frame_edit_counter), BindRef::new(&timeline.selected_layer));
        let selection           = SelectionModel::new();

        let size_binding        = bind(animation.size());

        FloModel {
            animation:          animation,
            tools:              tools,
            timeline:           timeline,
            frame_edit_counter: frame_edit_counter,
            frame:              frame,
            selection:          selection,

            size:               BindRef::from(size_binding.clone()),
            size_binding:       size_binding
        }
    }

    ///
    /// Retrieves the model for the drawing tools for this animation
    /// 
    pub fn tools(&self) -> &ToolModel<Anim> {
        &self.tools
    }

    ///
    /// Retrieves the model of the timeline for this animation
    /// 
    pub fn timeline(&self) -> &TimelineModel<Anim> {
        &self.timeline
    }

    ///
    /// Retrieves the frame model for this animation
    /// 
    pub fn frame(&self) -> &FrameModel {
        &self.frame
    }

    ///
    /// Retrieves the selection model for this animation
    /// 
    pub fn selection(&self) -> &SelectionModel {
        &self.selection
    }

    ///
    /// Retrieves the frame update binding for this animation
    /// 
    pub fn frame_update_count(&self) -> BindRef<u64> {
        BindRef::from(self.frame_edit_counter.clone())
    }
}

// Clone because for some reason #[derive(Clone)] does something weird
impl<Anim: Animation> Clone for FloModel<Anim> {
    fn clone(&self) -> FloModel<Anim> {
        FloModel {
            animation:          self.animation.clone(),
            tools:              self.tools.clone(),
            timeline:           self.timeline.clone(),
            frame_edit_counter: self.frame_edit_counter.clone(),
            frame:              self.frame.clone(),
            selection:          self.selection.clone(),

            size:               self.size.clone(),
            size_binding:       self.size_binding.clone()
        }
    }
}

impl<Anim: Animation> Animation for FloModel<Anim> {
    ///
    /// Retrieves the frame size of this animation
    /// 
    fn size(&self) -> (f64, f64) {
        self.animation.size()
    }

    ///
    /// Retrieves the length of this animation
    /// 
    fn duration(&self) -> Duration {
        self.animation.duration()
    }

    ///
    /// Retrieves the duration of a single frame
    /// 
    fn frame_length(&self) -> Duration {
        self.animation.frame_length()
    }

    ///
    /// Retrieves the IDs of the layers in this object
    /// 
    fn get_layer_ids(&self) -> Vec<u64> {
        self.animation.get_layer_ids()
    }

    ///
    /// Retrieves the layer with the specified ID from this animation
    /// 
    fn get_layer_with_id<'a>(&'a self, layer_id: u64) -> Option<Box<dyn 'a+Deref<Target=dyn 'a+Layer>>> {
        self.animation.get_layer_with_id(layer_id)
    }

    ///
    /// Retrieves the total number of items that have been performed on this animation
    /// 
    fn get_num_edits(&self) -> usize {
        self.animation.get_num_edits()
    }

    ///
    /// Reads from the edit log for this animation
    /// 
    fn read_edit_log<'a>(&'a self, range: Range<usize>) -> Box<dyn 'a+Stream<Item=AnimationEdit, Error=()>> {
        self.animation.read_edit_log(range)
    }

    ///
    /// Supplies a reference which can be used to find the motions associated with this animation
    /// 
    fn motion<'a>(&'a self) -> &'a dyn AnimationMotion {
        self
    }
}

impl<Anim: Animation> AnimationMotion for FloModel<Anim> {
    ///
    /// Assigns a new unique ID for creating a new motion
    /// 
    /// (This ID will not have been used so far and will not be used again)
    /// 
    fn assign_motion_id(&self) -> ElementId {
        self.animation.motion().assign_motion_id()
    }
    
    ///
    /// Retrieves a stream containing all of the motions in a particular time range
    /// 
    fn get_motion_ids(&self, when: Range<Duration>) -> Box<dyn Stream<Item=ElementId, Error=()>> {
        self.animation.motion().get_motion_ids(when)
    }

    ///
    /// Retrieves the IDs of the motions attached to a particular element
    /// 
    fn get_motions_for_element(&self, element_id: ElementId) -> Vec<ElementId> {
        self.animation.motion().get_motions_for_element(element_id)
    }

    ///
    /// Retrieves the IDs of the elements attached to a particular motion
    /// 
    fn get_elements_for_motion(&self, motion_id: ElementId) -> Vec<ElementId> {
        self.animation.motion().get_elements_for_motion(motion_id)
    }

    ///
    /// Retrieves the motion with the specified ID
    /// 
    fn get_motion(&self, motion_id: ElementId) -> Option<Motion> {
        self.animation.motion().get_motion(motion_id)
    }
}

///
/// Sink used to send data to the animation
/// 
struct FloModelSink<TargetSink, ProcessingFn> {
    /// Function called on every start send
    processing_fn: ProcessingFn,

    /// Sink where requests should be forwarded to 
    target_sink: TargetSink
}

impl<TargetSink, ProcessingFn> FloModelSink<TargetSink, ProcessingFn> {
    ///
    /// Creates a new model sink
    /// 
    pub fn new(target_sink: TargetSink, processing_fn: ProcessingFn) -> FloModelSink<TargetSink, ProcessingFn> {
        FloModelSink {
            processing_fn:  processing_fn,
            target_sink:    target_sink
        }
    }
}

impl<TargetSink: Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>, ProcessingFn: FnMut(&Vec<AnimationEdit>) -> ()> Sink for FloModelSink<TargetSink, ProcessingFn> {
    type SinkItem   = Vec<AnimationEdit>;
    type SinkError  = ();

    fn start_send(&mut self, item: Vec<AnimationEdit>) -> StartSend<Vec<AnimationEdit>, ()> {
        (self.processing_fn)(&item);

        self.target_sink.start_send(item)
    }

    fn poll_complete(&mut self) -> Poll<(), ()> {
        self.target_sink.poll_complete()
    }
}

impl<Anim: Animation+EditableAnimation> EditableAnimation for FloModel<Anim> {
    ///
    /// Retrieves a sink that can be used to send edits for this animation
    /// 
    /// Edits are supplied as groups (stored in a vec) so that it's possible to ensure that
    /// a set of related edits are performed atomically
    /// 
    fn edit(&self) -> Box<dyn Sink<SinkItem=Vec<AnimationEdit>, SinkError=()>+Send> {
        // Edit the underlying animation
        let animation_edit  = self.animation.edit();

        // Borrow the bits of the viewmodel we can change
        let frame_edit_counter  = self.frame_edit_counter.clone();
        let mut size_binding    = self.size_binding.clone();

        // Pipe the edits so they modify the model as a side-effect
        let model_edit          = FloModelSink::new(animation_edit, move |edits: &Vec<AnimationEdit>| {
            use self::AnimationEdit::*;
            use self::LayerEdit::*;

            // Update the viewmodel based on the edits that are about to go through
            let mut advance_edit_counter = false;

            for edit in edits.iter() {
                match edit {
                    SetSize(width, height) => {
                        size_binding.set((*width, *height));
                        advance_edit_counter = true;
                    },

                    AddNewLayer(_)              |
                    RemoveLayer(_)              |
                    Element(_, _)               |
                    Motion(_, _)                |
                    Layer(_, Paint(_, _))       => {
                        advance_edit_counter = true;
                    }

                    Layer(_, AddKeyFrame(_))    |
                    Layer(_, RemoveKeyFrame(_)) => {
                        ()
                    },
                }
            }

            // Advancing the frame edit counter causes any animation frames to be regenerated
            if advance_edit_counter {
                frame_edit_counter.clone().set(frame_edit_counter.get()+1);
            }
        });

        Box::new(model_edit)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use flo_animation::inmemory::*;
    use futures::executor;

    #[test]
    fn size_command_updates_size_binding() {
        let model = FloModel::new(InMemoryAnimation::new());

        // Initial size is 1980x1080
        assert!(model.size()        == (1980.0, 1080.0));
        assert!(model.size.get()    == (1980.0, 1080.0));

        // Change to 800x600
        {
            let mut edit_log = executor::spawn(model.edit());
            edit_log.wait_send(vec![AnimationEdit::SetSize(800.0, 600.0)]).unwrap();
        }

        // Binding should get changed by this edit
        assert!(model.size()        == (800.0, 600.0));
        assert!(model.size.get()    == (800.0, 600.0));
    }
}