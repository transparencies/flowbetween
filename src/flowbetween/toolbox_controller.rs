use super::tools::*;
use super::style::*;
use super::viewmodel::*;

use ui::*;
use binding::*;
use animation::*;

use std::sync::*;

///
/// The toolbox controller allows the user to pick which tool they
/// are using to edit the canvas
///
pub struct ToolboxController<Anim: Animation> {
    view_model:         Arc<DynamicViewModel>,
    ui:                 Arc<Bound<Control>>,
    images:             Arc<ResourceManager<Image>>,
    anim_view_model:    AnimationViewModel<Anim>
}

impl<Anim: 'static+Animation> ToolboxController<Anim> {
    ///
    /// Creates a new toolbox controller
    /// 
    pub fn new(anim_view_model: &AnimationViewModel<Anim>) -> ToolboxController<Anim> {
        // Create the viewmodel
        let viewmodel = Arc::new(DynamicViewModel::new());

        // There's a 'SelectedTool' key that describes the currently selected tool
        viewmodel.set_property("SelectedTool", PropertyValue::String("Pencil".to_string()));

        // Make sure that the tool selected in this controller matches the one in the main view model
        anim_view_model.tools().choose_tool_with_name(&viewmodel.get_property("SelectedTool").get().string().unwrap_or("".to_string()));

        // Some images for the root controller
        let images  = Arc::new(Self::create_images());

        // Set up the tools
        let ui = Self::create_ui(anim_view_model.tools().tool_sets(), Arc::clone(&viewmodel), Arc::clone(&images));

        // TODO: when the current tool is updated in the view model, update the selected tool here
        // TODO: also want a 'temporary' tool (like the eraser when it's in use, for example)

        ToolboxController {
            view_model:         viewmodel,
            ui:                 ui,
            anim_view_model:    anim_view_model.clone(),
            images:             images
        }
    }

    ///
    /// Creates the UI binding
    /// 
    fn create_ui(tool_sets: Binding<Vec<Arc<ToolSet<Anim>>>>, viewmodel: Arc<DynamicViewModel>, images: Arc<ResourceManager<Image>>) -> Arc<Bound<Control>> {
        Arc::new(computed(move || {
            // Convert the tool sets into tools (with separators between each individual set)
            let tools_for_sets: Vec<Control> = tool_sets.get().iter()
                .map(|toolset| {
                    let tools: Vec<Control> = toolset.tools().iter()
                        .map(|tool| Self::make_tool(&tool.tool_name(), &viewmodel, images.get_named_resource(&tool.image_name())))
                        .collect();
                    
                    tools
                }).fold(vec![], |mut result, new_items| {
                    // Separator between toolsets after the first set
                    if result.len() > 0 { result.push(Self::make_separator()); }

                    // Add the new items
                    result.extend(new_items.into_iter());

                    result
                });

            // Put the controls into a container
            Control::container()
                .with(Bounds::fill_all())
                .with(ControlAttribute::Background(TOOLS_BACKGROUND))
                .with(tools_for_sets)
        }))
    }

    ///
    /// Creates the image resources for this controller 
    ///
    fn create_images() -> ResourceManager<Image> {
        let images  = ResourceManager::new();

        // Load the tool images
        let select  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/select.svg")));
        let adjust  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/adjust.svg")));
        let pan     = images.register(svg_static(include_bytes!("../../static_files/svg/tools/pan.svg")));

        let pencil  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/pencil.svg")));
        let ink     = images.register(svg_static(include_bytes!("../../static_files/svg/tools/ink.svg")));
        let eraser  = images.register(svg_static(include_bytes!("../../static_files/svg/tools/eraser.svg")));

        // Assign names to them
        images.assign_name(&select, "select");
        images.assign_name(&adjust, "adjust");
        images.assign_name(&pan, "pan");

        images.assign_name(&pencil, "pencil");
        images.assign_name(&ink, "ink");
        images.assign_name(&eraser, "eraser");

        images
    }

    ///
    /// Creates a separator between controls
    /// 
    fn make_separator() -> Control {
        Control::empty()
            .with(Bounds::next_vert(12.0))
    }

    ///
    /// Creates a new tool control
    ///
    fn make_tool(name: &str, viewmodel: &DynamicViewModel, image: Option<Resource<Image>>) -> Control {
        use ui::ControlAttribute::*;
        use ui::ActionTrigger::*;

        // Decide if this is the selected tool
        let selected_tool   = viewmodel.get_property("SelectedTool");

        // The tool has a '-selected' binding that we use to cause it to highlight
        let compare_name            = String::from(name);
        let selected_property_name  = format!("{}-selected", name);

        // When the selected tool is set to the name of this tool, the selected property should be set to true
        viewmodel.set_computed(&selected_property_name, move || {
            let selected_tool = selected_tool.get().string().unwrap_or(String::from(""));
            PropertyValue::Bool(selected_tool == compare_name)
        });

        // The control is just a button
        Control::button()
            .with((Click, name))
            .with(Selected(Property::Bind(selected_property_name)))
            .with(Bounds::next_vert(48.0))
            .with(vec![
                Control::empty()
                    .with(Bounds::fill_all())
                    .with(image)
            ])
    }
}

impl<Anim: 'static+Animation> Controller for ToolboxController<Anim> {
    fn ui(&self) -> Arc<Bound<Control>> {
        self.ui.clone()
    }

    fn action(&self, action_id: &str, _action_parameter: &ActionParameter) {
        // Set the selected tool in the UI view model
        self.view_model.set_property("SelectedTool", PropertyValue::String(String::from(action_id)));

        // Update the animation view model with the newly selected tool
        self.anim_view_model.tools().choose_tool_with_name(action_id);
    }

    fn get_viewmodel(&self) -> Arc<ViewModel> {
        self.view_model.clone()
    }

    fn get_image_resources(&self) -> Option<Arc<ResourceManager<Image>>> {
        Some(self.images.clone())
    }
}
