use crate::scenery::document::*;
use super::document::*;

use flo_draw::*;
use flo_draw::draw_scene::*;
use flo_draw::canvas::scenery::*;
use flo_scene::*;
use flo_scene::programs::*;
use flo_binding::*;
use futures::prelude::*;
use serde::*;

use std::sync::*;
use std::collections::*;

///
/// Runs the main flowbetween program
///
pub async fn flowbetween(scene: Arc<Scene>, events: InputStream<FlowBetween>, context: SceneContext) {
    let mut events      = events;
    let mut documents   = HashMap::new();

    while let Some(evt) = events.next().await {
        use FlowBetween::*;

        match evt {
            CreateEmptyDocument(document_id) => {
                // Create a program ID for the document program
                let document_program_id = SubProgramId::new();

                // Create the document program
                create_empty_document(Arc::clone(&scene), document_program_id, &context).await;

                // Store as in the list of known document programs
                documents.insert(document_id, document_program_id);
            }
        }
    }
}

///
/// Commands for controlling the main flowbetween program
///
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlowBetween {
    CreateEmptyDocument(DocumentId),
}

impl SceneMessage for FlowBetween { 
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::flowbetween").into()
    }
}

///
/// Program that relays events originating from the window in the app scene to the main document in the document scene
///
async fn event_relay_program(drawing_events: impl Unpin + Send + Sink<DocumentRequest>, document_program_id: SubProgramId, input: InputStream<DrawEvent>, context: SceneContext) {
    let mut input           = input;
    let mut drawing_events  = drawing_events;
    let mut scale           = 1.0;
    let mut size            = (0.0, 0.0);

    while let Some(event) = input.next().await {
        // Interpret some special events
        match &event {
            DrawEvent::Resize(w, h) => {
                // Send a resize request for the resize event
                drawing_events.send(DocumentRequest::Resize((w/scale) as _, (h/scale) as _)).await.ok();
                size = (*w, *h);
            }

            DrawEvent::Scale(new_scale) => {
                // Send a resize event when the scale changes
                if *new_scale != 0.0 && *new_scale != scale {
                    let (w, h) = size;
                    scale = *new_scale;

                    drawing_events.send(DocumentRequest::Resize((w/scale) as _, (h/scale) as _)).await.ok();
                }
            }

            DrawEvent::Closed => {
                // Tell the document to close down when the close request arrives
                drawing_events.send(DocumentRequest::Close).await.ok();

                // Also close down the main document scene when the window is closed
                context.send_message(SceneControl::Close(document_program_id)).await.ok();
            }

            _ => { }
        }

        // Send the event as a normal event
        if drawing_events.send(DocumentRequest::Event(event)).await.is_err() {
            break;
        }
    }
}

///
/// Program that runs in the document scene and sends DrawingWindowRequests out to the application scene (where the actual window lives)
///
async fn drawing_relay_program(drawing_requests: OutputSink<DrawingWindowRequest>, input: InputStream<DrawingWindowRequest>, _context: SceneContext) {
    let mut drawing_requests = drawing_requests;
    let mut input            = input;

    while let Some(request) = input.next().await {
        // We can't wire up 'SendEvents' or similar messages here as they'll send their responses to the app scene, so we ignore them for now
        if let DrawingWindowRequest::SendEvents(_) = &request {
            // TODO: could start an event relay program here instead (we'd have to manage it and stop it when we're done though)
            continue;
        }

        // Pass the event on to the application scene
        let maybe_err = drawing_requests.send(request).await;

        if !maybe_err.is_ok() {
            break;
        }
    }
}

///
/// Creates an empty document in the context
///
async fn create_empty_document(scene: Arc<Scene>, document_program_id: SubProgramId, context: &SceneContext) {
    let properties = WindowProperties::from(&());

    // Create a window for this document
    let render_window_program_id   = SubProgramId::new();
    let drawing_window_program_id  = SubProgramId::new();
    let event_relay_program_id     = SubProgramId::new();

    create_render_window_sub_program(&scene, render_window_program_id, properties.size().get()).unwrap();
    create_drawing_window_program(&scene, drawing_window_program_id, render_window_program_id).unwrap();

    // Each document runs in its own isolated scene (which lets us run subprograms in the scene with their own IDs + shut everything down cleanly when we're done)
    let document_scene = Arc::new(Scene::default());

    // Add a subprogram in the document scene that relays drawing instructions from the document scene to the drawing window (as the window runs in the 'main' scene, we don't have access here)
    let drawing_requests = context.send::<DrawingWindowRequest>(drawing_window_program_id).unwrap();
    document_scene.add_subprogram(subprogram_window(), move |input, context| drawing_relay_program(drawing_requests, input, context), 100);

    // Allow drawing requests to be sent directly to the window
    let drawing_request_filter = FilterHandle::for_filter(|drawing_requests| drawing_requests.map(|req| DrawingWindowRequest::Draw(req)));
    document_scene.connect_programs((), StreamTarget::Filtered(drawing_request_filter.clone(), subprogram_window()), StreamId::with_message_type::<DrawingRequest>()).unwrap();

    // Document program sends to the window (it also is the default target for drawing requests for the scene)
    document_scene.connect_programs(subprogram_flowbetween_document(), StreamTarget::Filtered(drawing_request_filter, subprogram_window()), StreamId::with_message_type::<DrawingRequest>()).unwrap();

    // Start the main program within the document scene
    let document_scene_clone    = Arc::clone(&document_scene);
    let app_scene_control       = context.send(()).unwrap();
    document_scene.add_subprogram(subprogram_flowbetween_document(), move |input, context| async move {
        // We want to close the document program in the app scene when the program in the document scene finishes
        let mut app_scene_control = app_scene_control;

        // Run the document program
        flowbetween_document(document_scene_clone, input, context).await;

        // Shut down the app document program when the document scene's main program stops
        app_scene_control.send(SceneControl::Close(document_program_id)).await.ok();
    }, 20);

    // Add a subprogram to the app scene that relays events from the window to the document scene
    let drawing_events = document_scene.send_to_scene(()).unwrap();
    scene.add_subprogram(event_relay_program_id, move |input, context| event_relay_program(drawing_events, document_program_id, input, context), 20);

    context.send(drawing_window_program_id).unwrap()
        .send(DrawingWindowRequest::SendEvents(event_relay_program_id)).await.ok();

    // Run the document scene in its own subprogram (within the app)
    scene.add_subprogram(document_program_id, move |input, context| async move {
        // Run the scene
        document(document_scene, input, context.clone()).await;

        // When the scene is finished, stop all of the subprograms running in the main scene
        context.send_message(SceneControl::Close(event_relay_program_id)).await.ok();
        context.send_message(SceneControl::Close(drawing_window_program_id)).await.ok();
        context.send_message(SceneControl::Close(render_window_program_id)).await.ok();
    }, 1);
}
