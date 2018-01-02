use super::path::*;
use super::element::*;

use super::super::brush::*;
use super::super::brush_properties::*;

use canvas::*;

use std::sync::*;
use std::time::Duration;

///
/// Element representing a brush stroke
///
#[derive(Clone)]
pub struct BrushElement {
    /// When this element is drawn relative to the start of the frame
    appearance_time: Duration,

    // The properties for this element (TODO: group a bunch of brush elements with the same properties)
    properties: BrushProperties,

    /// The path taken by this brush stroke
    points: Vec<BrushPoint>,

    /// The brush that will be used for this brush stroke
    brush: Arc<Brush>
}

impl BrushElement {
    ///
    /// Begins a new brush stroke at a particular position
    /// 
    pub fn new(brush: &Arc<Brush>, appearance_time: Duration, start_pos: BrushPoint, properties: &BrushProperties) -> BrushElement {
        BrushElement {
            properties:         *properties,
            appearance_time:    appearance_time,
            points:             vec![start_pos],
            brush:              Arc::clone(brush)
        }
    }

    ///
    /// Adds a new brush point to this item
    /// 
    pub fn add_point(&mut self, point: BrushPoint) {
        self.points.push(point);
    }

    ///
    /// Updates the appearance time of this item
    /// 
    pub fn set_appearance_time(&mut self, new_time: Duration) {
        self.appearance_time = new_time;
    }

    ///
    /// Retrieves the points in this brush element
    /// 
    pub fn points<'a>(&'a self) -> &'a Vec<BrushPoint> {
        &self.points
    }
}

impl VectorElement for BrushElement {
    fn appearance_time(&self) -> Duration {
        self.appearance_time
    }

    fn path(&self) -> Path {
        let move_element    = vec![PathElement::Move(PathPoint::from(self.points[0]))];
        let line_elements   = self.points.iter().skip(1).map(|point| PathElement::Line(PathPoint::from(point)));

        Path {
            elements: move_element.into_iter().chain(line_elements).collect()
        }
    }

    fn render(&self, gc: &mut GraphicsPrimitives) {
        // TODO: find a way to only prepare to render once per brush
        self.brush.prepare_to_render(gc, &self.properties);
        self.brush.render_brush(gc, &self.points)
    }
}