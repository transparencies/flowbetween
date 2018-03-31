use super::property_action::*;
use super::super::gtk_action::*;

use flo_ui::*;

pub type PropertyWidgetAction = PropertyAction<GtkWidgetAction>;

///
/// Trait implemented by things that can be converted to GTK widget actions
/// 
pub trait ToGtkActions {
    ///
    /// Converts this itme to a set of GtkWIdgetActions required to render it to a GTK widget
    /// 
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction>;
}

impl ToGtkActions for ControlAttribute {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::ControlAttribute::*;

        match self {
            &BoundingBox(ref bounds)                => vec![ WidgetLayout::BoundingBox(bounds.clone()).into() ].into_actions(),
            &ZIndex(zindex)                         => vec![ WidgetLayout::ZIndex(zindex).into() ].into_actions(),
            &Padding((left, top), (right, bottom))  => vec![ WidgetLayout::Padding((left, top), (right, bottom)).into() ].into_actions(),
            
            &Text(ref text)                         => vec![ PropertyAction::from_property(text.clone(), |text| WidgetContent::SetText(text.to_string()).into()) ],

            &FontAttr(ref font)                     => font.to_gtk_actions(),
            &StateAttr(ref state)                   => state.to_gtk_actions(),
            &PopupAttr(ref popup)                   => popup.to_gtk_actions(),
            &AppearanceAttr(ref appearance)         => appearance.to_gtk_actions(),
            &ScrollAttr(ref scroll)                 => scroll.to_gtk_actions(),

            &Id(ref id)                             => vec![ GtkWidgetAction::Content(WidgetContent::AddClass(id.clone())) ].into_actions(),
            &Action(ref trigger, ref action_name)   => unimplemented!(),
            &Canvas(ref canvas)                     => unimplemented!(),

            // The GTK layout doesn't need to know the controller
            &Controller(ref _controller_name)       => vec![],

            // Subcomponents are added elsewhere: we don't assign them here
            &SubComponents(ref _components)         => vec![]
        }
    }
}

impl ToGtkActions for State {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::State::*;

        match self {
            &Selected(ref selected)     => vec![ PropertyAction::from_property(selected.clone(), |value| WidgetState::SetSelected(value.to_bool().unwrap_or(false)).into()) ],
            &Badged(ref badged)         => vec![ PropertyAction::from_property(badged.clone(), |value| WidgetState::SetBadged(value.to_bool().unwrap_or(false)).into()) ],
            &Value(ref value)           => vec![ PropertyAction::from_property(value.clone(), |value| WidgetState::SetValueFloat(value.to_f32().unwrap_or(0.0)).into()) ],
            &Range((ref min, ref max))  => vec![ 
                PropertyAction::from_property(min.clone(), |min| WidgetState::SetRangeMin(min.to_f32().unwrap_or(0.0)).into()),
                PropertyAction::from_property(max.clone(), |max| WidgetState::SetRangeMax(max.to_f32().unwrap_or(0.0)).into()) 
            ]
        }
    }
}

impl ToGtkActions for Popup {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::Popup::*;

        match self {
            &IsOpen(ref is_open)        => unimplemented!(),
            &Direction(ref direction)   => unimplemented!(),
            &Size(width, height)        => unimplemented!(),
            &Offset(u32)                => unimplemented!()
        }
    }
}

impl ToGtkActions for Font {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        vec![ self.clone().into() ].into_actions()
    }
}

impl ToGtkActions for Appearance {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        vec![ self.clone().into() ].into_actions()
    }
}

impl ToGtkActions for Scroll {
    fn to_gtk_actions(&self) -> Vec<PropertyWidgetAction> {
        use self::Scroll::*;

        match self {
            &MinimumContentSize(width, height)      => unimplemented!(),
            &HorizontalScrollBar(ref visibility)    => unimplemented!(),
            &VerticalScrollBar(ref visibility)      => unimplemented!(),
            &Fix(ref axis)                          => unimplemented!()
        }
    }
}
