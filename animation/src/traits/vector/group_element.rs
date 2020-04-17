use super::vector::*;
use super::properties::*;
use super::control_point::*;
use super::vector_element::*;
use super::path_conversion_options::*;
use super::super::edit::*;
use super::super::path::*;
use super::super::motion::*;
use super::super::group_type::*;

use flo_canvas::*;
use flo_curves::bezier::path::*;

use std::sync::*;
use std::time::Duration;

///
/// Represents an element consisting of a group of other elements
///
#[derive(Clone, Debug)]
pub struct GroupElement {
    /// The ID assigned to this element
    id: ElementId,

    /// The type of this group
    group_type: GroupType,

    /// The elements that make up this group
    grouped_elements: Arc<Vec<Vector>>,

    /// The hint path if one is set
    hint_path: Option<Arc<Vec<Path>>>
}

impl GroupElement {
    ///
    /// Creates a new group from a set of elements
    ///
    pub fn new(id: ElementId, group_type: GroupType, grouped_elements: Arc<Vec<Vector>>) -> GroupElement {
        GroupElement {
            id:                 id,
            group_type:         group_type,
            grouped_elements:   grouped_elements,
            hint_path:          None
        }
    }

    ///
    /// Retrieves the type of this group
    ///
    pub fn group_type(&self) -> GroupType {
        self.group_type
    }

    ///
    /// Sets a hint path for this element
    ///
    /// For certain group types that generate an output path with a single property (eg, GroupType::Added), this path will be
    /// used instead of recomputing the path arithmetic operation represented by this group
    ///
    pub fn set_hint_path(&mut self, hint_path: Arc<Vec<Path>>) {
        self.hint_path = Some(hint_path);
    }

    ///
    /// Retrieves the hint path if one is set
    ///
    pub fn hint_path(&self) -> Option<Arc<Vec<Path>>> {
        self.hint_path.as_ref().map(|path| Arc::clone(path))
    }

    ///
    /// Renders the contents of this group in 'normal' mode
    ///
    fn render_normal(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        // Properties update internally to the group
        let mut properties = Arc::new(properties.clone());

        for elem in self.grouped_elements.iter() {
            properties = elem.update_properties(properties);
            properties.render(gc, elem.clone(), when);
        }
    }

    ///
    /// Returns the added path for this element
    ///
    fn added_path(&self, properties: &VectorProperties) -> Vec<Path> {
        if let Some(hint_path) = self.hint_path.as_ref() {
            // If a hint path has been set we can use this as the short-circuit for this path
            (**hint_path).clone()
        } else {
            // Get the paths for this rendering
            let paths = self.grouped_elements.iter()
                .flat_map(|elem| elem.to_path(properties, PathConversion::RemoveInteriorPoints))
                .collect::<Vec<_>>();

            // Render if there are more than one path
            if paths.len() > 0 {
                // Add the paths into a single path
                let paths = path_add_chain::<_, Path>(&paths, 0.01);
                vec![Path::from_paths(&paths)]
            } else {
                vec![]
            }
        }
    }

    ///
    /// Renders the contents of this group in 'added' mode
    ///
    fn render_added(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties) {
        let paths = self.added_path(properties);

        gc.draw_list(properties.brush.prepare_to_render(&properties.brush_properties));
        paths.into_iter()
            .for_each(|path| gc.draw_list(properties.brush.render_path(&properties.brush_properties, &path)));
    }

    ///
    /// The number of elements in this group
    ///
    pub fn num_elements(&self) -> usize {
        self.grouped_elements.len()
    }

    ///
    /// Retrieves the elements in this group
    ///
    pub fn elements(&self) -> impl Iterator<Item=&Vector> {
        self.grouped_elements.iter()
    }

    ///
    /// Creates a new version of this group element with an alternative set of elements attached
    ///
    pub fn with_elements<Elements: IntoIterator<Item=Vector>>(&self, elements: Elements) -> GroupElement {
        GroupElement::new(self.id, self.group_type, Arc::new(elements.into_iter().collect()))
    }
}

impl VectorElement for GroupElement {
    ///
    /// The ID of this element
    ///
    fn id(&self) -> ElementId {
        self.id
    }

    ///
    /// Modifies this element to have a new ID
    ///
    fn set_id(&mut self, new_id: ElementId) {
        self.id = new_id
    }

    ///
    /// Retrieves the paths for this element, if there are any
    ///
    fn to_path(&self, properties: &VectorProperties, options: PathConversion) -> Option<Vec<Path>> {
        // With the added path type we can assume that the interior points are already removed so there's no need to apply the options
        match self.group_type {
            GroupType::Normal   => Some(self.grouped_elements.iter().flat_map(|elem| elem.to_path(properties, options)).flatten().collect()),
            GroupType::Added    => Some(self.added_path(properties))
        }
    }

    ///
    /// Renders this vector element
    ///
    fn render(&self, gc: &mut dyn GraphicsPrimitives, properties: &VectorProperties, when: Duration) {
        match self.group_type {
            GroupType::Normal   => self.render_normal(gc, properties, when),
            GroupType::Added    => self.render_added(gc, properties)
        }
    }

    ///
    /// Returns the properties to use for future elements
    ///
    fn update_properties(&self, properties: Arc<VectorProperties>) -> Arc<VectorProperties> {
        // Groups do not update properties
        properties
    }

    ///
    /// Returns a new element that is this element transformed along a motion at a particular moment
    /// in time.
    ///
    fn motion_transform(&self, motion: &Motion, when: Duration) -> Vector {
        let new_elements = self.grouped_elements.iter()
            .map(|old_elem| old_elem.motion_transform(motion, when))
            .collect();

        Vector::Group(GroupElement::new(self.id, self.group_type, Arc::new(new_elements)))
    }

    ///
    /// Fetches the control points for this element
    ///
    fn control_points(&self) -> Vec<ControlPoint> {
        vec![]
    }

    ///
    /// Creates a new vector element from this one with the control points updated to the specified set of new values
    ///
    /// The vector here specifies the updated position for each control point in control_points
    ///
    fn with_adjusted_control_points(&self, _new_positions: Vec<(f32, f32)>) -> Vector {
        Vector::Group(self.clone())
    }
}
