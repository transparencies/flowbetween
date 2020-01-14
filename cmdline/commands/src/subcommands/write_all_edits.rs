use super::super::state::*;
use super::super::error::*;
use super::super::output::*;

use flo_stream::*;
use ::desync::*;

use futures::prelude::*;
use std::sync::*;

///
/// The read_all_edits command loads all of the edits from the input animation and stores them in the state buffer
///
pub fn write_all_edits<'a>(output: &'a mut Publisher<FloCommandOutput>, state: &'a mut CommandState) -> impl Future<Output=Result<(), CommandError>>+Send+'a {
    async move {
        // Load the output animation
        let output_anim     = Desync::new(state.output_animation());

        // Write the edit buffer to it
        let mut edit_output = output.republish();
        let edits           = state.edit_buffer().clone();
        let edits           = output_anim.future(move |output_anim| {
            // Write edits one at a time to the output animation
            async move {
                edit_output.publish(FloCommandOutput::StartTask("Read edit log".to_string())).await;

                // Write the edits one at a time and update on progress
                let mut edit_sink = output_anim.edit();

                for edit_index in 0..edits.len() {
                    let next_edit = edits[edit_index].clone();
                    edit_sink.publish(Arc::new(vec![next_edit])).await;

                    edit_output.publish(FloCommandOutput::TaskProgress(edit_index as f64, edits.len() as f64)).await;
                }
                edit_output.publish(FloCommandOutput::FinishTask).await;

                let finish_message = format!("Wrote {} edits to the output animation", edits.len());
                edit_output.publish(FloCommandOutput::Message(finish_message)).await;

                edits
            }.boxed()
        }).await;

        // Update the edit buffer
        *state = state.set_edit_buffer(edits.unwrap());

        Ok(())
    }
}
