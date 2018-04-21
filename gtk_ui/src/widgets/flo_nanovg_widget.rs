use super::widget::*;
use super::basic_widget::*;
use super::super::gtk_thread::*;
use super::super::gtk_action::*;

use flo_nanovg_canvas::*;

use gtk;
use gtk::prelude::*;
use gl;
use nanovg;

use std::rc::*;
use std::cell::*;

///
/// NanoVG core data, shared with event handlers
/// 
struct NanoVgCore {
    /// The context, if it exists
    context: Option<nanovg::Context>,

    // The layers in this core
    layers: Option<NanoVgLayers>
}

///
/// Uses NanoVG to draw using OpenGL on a widget
/// 
pub struct FloNanoVgWidget {
    /// The ID of this widget
    id: WidgetId,

    /// The GTK GLArea widget (needs to be explicitly retained to avoid random self-destruction)
    _gl_widget: gtk::GLArea,

    /// The widget that the rest of the code will deal with
    as_widget: gtk::Widget
}

impl FloNanoVgWidget {
    ///
    /// Creates a new NanoVG widget with a particular GL area as the target
    /// 
    pub fn new<W: Clone+Cast+IsA<gtk::GLArea>>(widget_id: WidgetId, widget: W) -> FloNanoVgWidget {
        // Fetch the GL widget and its widget representation
        let gl_widget = widget.upcast::<gtk::GLArea>();
        let as_widget = gl_widget.clone().upcast::<gtk::Widget>();

        // Create the core data
        let core = NanoVgCore {
            layers:     None,
            context:    None
        };
        let core = Rc::new(RefCell::new(core));

        // Configure the GL area
        gl_widget.set_has_alpha(true);
        gl_widget.set_has_stencil_buffer(true);

        // Simple realize event
        {
            let core = Rc::clone(&core);
            gl_widget.connect_realize(move |gl_widget| {
                let mut core = core.borrow_mut();

                // Set the context
                gl_widget.make_current();

                let random_framebuffer = FrameBuffer::new(200, 300);

                // Create the nanovg context
                let context     = nanovg::ContextBuilder::new()
                    .stencil_strokes()
                    .antialias()
                    .build()
                    .expect("Failed to create NanoVG context");
                core.context    = Some(context);

                // ... and layers
                let allocation  = gl_widget.get_allocation();
                let scale       = gl_widget.get_scale_factor() as f32;
                let layers      = Some(NanoVgLayers::new(Self::get_viewport(&gl_widget.clone().upcast::<gtk::Widget>(), &allocation), scale));
                core.layers     = layers;
            });
        }

        // Simple rendering to test out our widget
        {
            let core = Rc::clone(&core);
            gl_widget.connect_render(move |gl_widget, _ctxt| { 
                let core        = core.borrow();
                let allocation  = gl_widget.get_allocation();
                let context     = core.context.as_ref().unwrap();
                let scale       = gl_widget.get_scale_factor();

                // Prepare to render
                unsafe {
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                    gl::Viewport(0, 0, allocation.width*scale, allocation.height*scale);
                }

                context.frame((allocation.width, allocation.height), scale as f32, |frame| {
                    frame.path(|path| {
                        path.rect((100.0, 100.0), (1980.0-200.0, 1080.0-200.0));
                        path.fill(nanovg::Color::new(0.5, 0.5, 0.8, 0.5), Default::default());
                    }, nanovg::PathOptions { clip: nanovg::Clip::None, composite_operation: nanovg::CompositeOperation::Basic(nanovg::BasicCompositeOperation::SourceOver), alpha: 1.0, transform: None });

                    frame.path(|path| {
                        path.circle((1980.0/2.0, 1080.0/2.0), 100.0);
                        path.fill(nanovg::Color::new(0.8, 0.5, 0.2, 1.0), Default::default());
                    }, nanovg::PathOptions { clip: nanovg::Clip::None, composite_operation: nanovg::CompositeOperation::Basic(nanovg::BasicCompositeOperation::SourceOver), alpha: 1.0, transform: None });
                });

                Inhibit(true)
            });
        }

        // Generate the result
        FloNanoVgWidget {
            id:         widget_id,
            _gl_widget: gl_widget,
            as_widget:  as_widget
        }
    }

    ///
    /// Retrieves the viewport for a canvas
    /// 
    fn get_viewport(drawing_area: &gtk::Widget, allocation: &gtk::Allocation) -> NanoVgViewport {
        // The scale factor is used to ensure we get a 1:1 pixel ratio for our drawing area
        let scale_factor = drawing_area.get_scale_factor();

        // Search for a scrollable parent to base the viewport upon
        let mut scrollable  = None;
        let mut parent      = Some(drawing_area.clone().upcast::<gtk::Widget>());
        while parent.is_some() && scrollable.is_none() {
            scrollable  = parent.clone().and_then(|parent| parent.dynamic_cast::<gtk::Scrollable>().ok());
            parent      = parent.and_then(|parent| parent.get_parent());
        }

        // Generate a viewport
        let viewport = NanoVgViewport {
            width:              allocation.width.max(1) * scale_factor,
            height:             allocation.height.max(1) * scale_factor,
            viewport_x:         0,
            viewport_y:         0,
            viewport_width:     allocation.width.max(1) * scale_factor,
            viewport_height:    allocation.height.max(1) * scale_factor
        };

        // Clip to the scrollable region if there is one
        match scrollable {
            Some(scrollable)    => Self::clip_viewport_to_scrollable(viewport, &scrollable, drawing_area),
            None                => viewport
        }
    }

    ///
    /// Clips a viewport to only the portion visible in a scrollable area
    ///
    fn clip_viewport_to_scrollable(full_viewport: NanoVgViewport, scrollable: &gtk::Scrollable, drawing_area: &gtk::Widget) -> NanoVgViewport {
        // Scrollable must also be a widget
        let scrollable_widget = scrollable.clone().dynamic_cast::<gtk::Widget>().unwrap();

        // Will need to scale the coorindates
        let scale       = drawing_area.get_scale_factor();

        // Get the positions for the scrollable
        let hadjust     = scrollable.get_hadjustment().unwrap();
        let vadjust     = scrollable.get_vadjustment().unwrap();

        let hvalue      = hadjust.get_value() as i32;       // = left coordinate
        let hpagesize   = hadjust.get_page_size() as i32;   // = width

        let vvalue      = vadjust.get_value() as i32;       // = top coordinate
        let vpagesize   = vadjust.get_page_size() as i32;   // = height

        // TODO: this should really be '&&', maybe allowing for up to a certain size (we get a giant viewport in the timeline right now, so this isn't done)
        if full_viewport.viewport_width <= hpagesize*scale || full_viewport.viewport_height <= vpagesize*scale {
            // If the scroll region is larger than the viewport then just use the full viewport
            full_viewport
        } else {
            // Turn the values into coorindates on the scrolling area (note that translate_coordinates returns scaled coordinates for some reason)
            let (left, top) = scrollable_widget.translate_coordinates(drawing_area, hvalue, vvalue).unwrap();

            // TODO: if the page size is greater than the canvas size, we should probably trim to only the area covered by the actual canvas

            // Otherwise, adjust the viewport to the scroll values
            NanoVgViewport {
                width:              full_viewport.width,
                height:             full_viewport.height,
                viewport_x:         left,                   // Scaled by translate_coordinates
                viewport_y:         top,                    // Scaled by translate_coordinates
                viewport_width:     hpagesize * scale,
                viewport_height:    vpagesize * scale
            }
        }
    }
}

impl GtkUiWidget for FloNanoVgWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn process(&mut self, flo_gtk: &mut FloGtk, action: &GtkWidgetAction) {
        process_basic_widget_action(self, flo_gtk, action);
    }

    fn set_children(&mut self, _children: Vec<Rc<RefCell<GtkUiWidget>>>) {
        // NanoVG widgets cannot have child widgets
    }

    fn get_underlying<'a>(&'a self) -> &'a gtk::Widget {
        &self.as_widget
    }
}