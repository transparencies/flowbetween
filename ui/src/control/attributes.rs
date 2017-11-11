use super::bounds::*;
use super::control::*;
use super::actions::*;

use super::super::property::*;

///
/// Attribute attached to a control
///
#[derive(Clone, PartialEq)]
pub enum ControlAttribute {
    /// The bounding box for this control
    BoundingBox(Bounds),

    /// The text for this control
    Text(Property),

    /// Whether or not this control is selected
    Selected(Property),

    /// The unique ID for this control
    Id(String),

    /// Subcomponents of this control
    SubComponents(Vec<Control>),

    /// Specifies the controller that manages the subcomponents of this control
    Controller(String),

    ///
    /// When the specified action occurs for this item, send the event 
    /// denoted by the string to the controller
    ///
    Action(ActionTrigger, String)
}

impl ControlAttribute {
    ///
    /// The bounding box represented by this attribute
    ///
    pub fn bounding_box<'a>(&'a self) -> Option<&'a Bounds> {
        match self {
            &BoundingBox(ref bounds)    => Some(bounds),
            _                           => None
        }
    }

    ///
    /// The text represented by this attribute
    ///
    pub fn text<'a>(&'a self) -> Option<&'a Property> {
        match self {
            &Text(ref text) => Some(text),
            _               => None
        }
    }

    ///
    /// The ID represented by this attribute
    ///
    pub fn id<'a>(&'a self) -> Option<&'a String> {
        match self {
            &Id(ref id) => Some(id),
            _           => None
        }
    }

    ///
    /// The subcomponent represented by this attribute
    ///
    pub fn subcomponents<'a>(&'a self) -> Option<&'a Vec<Control>> {
        match self {
            &SubComponents(ref components)  => Some(components),
            _                               => None
        }
    }

    ///
    /// The controller represented by this attribute
    ///
    pub fn controller<'a>(&'a self) -> Option<&'a str> {
        match self {
            &Controller(ref controller) => Some(controller),
            _                           => None
        }
    }

    ///
    /// The action represented by this attribute
    ///
    pub fn action<'a>(&'a self) -> Option<(&'a ActionTrigger, &'a String)> {
        match self {
            &Action(ref trigger, ref action)    => Some((trigger, action)),
            _                                   => None
        }
    }

    pub fn selected<'a>(&'a self) -> Option<&'a Property> {
        match self {
            &Selected(ref is_selected)  => Some(is_selected),
            _                           => None
        }
    }

    ///
    /// Returns true if this attribute is different from another one
    /// (non-recursively, so this won't check subcomoponents)
    ///
    pub fn is_different_flat(&self, compare_to: &ControlAttribute) -> bool {
        match self {
            &BoundingBox(ref bounds)            => Some(bounds) == compare_to.bounding_box(),
            &Text(ref text)                     => Some(text) == compare_to.text(),
            &Id(ref id)                         => Some(id) == compare_to.id(),
            &Controller(ref controller)         => Some(controller.as_ref()) == compare_to.controller(),
            &Action(ref trigger, ref action)    => Some((trigger, action)) == compare_to.action(),
            &Selected(ref is_selected)          => Some(is_selected) == compare_to.selected(),

            // For the subcomponents we only care about the number as we don't want to recurse
            &SubComponents(ref components)  => Some(components.len()) == compare_to.subcomponents().map(|components| components.len())
        }
    }
}

use ControlAttribute::*;

///
/// Trait implemented by things that can be converted into control attributes
///
pub trait ToControlAttributes {
    fn attributes(&self) -> Vec<ControlAttribute>;
}

impl ToControlAttributes for ControlAttribute {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![self.clone()]
    }
}

impl<'a> ToControlAttributes for &'a str {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![Text(self.to_property())]
    }
}

impl ToControlAttributes for Bounds {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![BoundingBox(self.clone())]
    }
}

impl ToControlAttributes for (ActionTrigger, String) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![Action(self.0.clone(), self.1.clone())]
    }
}

impl ToControlAttributes for Vec<ControlAttribute> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        self.clone()
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes> ToControlAttributes for (A, B) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());

        res
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes, C: ToControlAttributes> ToControlAttributes for (A, B, C) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());

        res
    }
}

impl<A: ToControlAttributes, B: ToControlAttributes, C: ToControlAttributes, D: ToControlAttributes> ToControlAttributes for (A, B, C, D) {
    fn attributes(&self) -> Vec<ControlAttribute> {
        let mut res = self.0.attributes();
        res.append(&mut self.1.attributes());
        res.append(&mut self.2.attributes());
        res.append(&mut self.3.attributes());

        res
    }
}

impl ToControlAttributes for Vec<Control> {
    fn attributes(&self) -> Vec<ControlAttribute> {
        vec![SubComponents(self.iter().cloned().collect())]
    }
}
