use super::storage_api::*;

use ::desync::*;

use futures::prelude::*;
use futures::future;

use std::u64;
use std::sync::*;
use std::time::{Duration};
use std::collections::{HashMap};

///
/// Represents a key frame
///
struct InMemoryKeyFrameStorage {
    /// The time when this frame appears
    when: Duration,

    /// The IDs of the elements attached to this keyframe
    attached_elements: HashMap<i64, Duration>
}

///
/// Representation of a layer in memory
///
struct InMemoryLayerStorage {
    /// The properties for this layer
    properties: String,

    /// The keyframes of this layer
    keyframes: Vec<InMemoryKeyFrameStorage>
}

///
/// Indicates where an element is attached
///
struct ElementAttachment {
    layer_id:       u64,
    keyframe_time:  Duration
}

///
/// Representation of an animation in-memory
///
struct InMemoryStorageCore {
    /// The properties for the animation
    animation_properties: Option<String>,

    /// The edit log
    edit_log: Vec<String>,

    /// The definitions for each element
    elements: HashMap<i64, String>,

    /// The keyframes that an element is attached to
    element_attachments: HashMap<i64, Vec<ElementAttachment>>,

    /// The layers
    layers: HashMap<u64, InMemoryLayerStorage>
}

///
/// Provides an implementation of the storage API that stores its data in memory
///
pub struct InMemoryStorage {
    /// Where the data is stored for this object 
    storage: Arc<Desync<InMemoryStorageCore>>
}

impl InMemoryStorage {
    ///
    /// Creates a new in-memory storage for an animation
    ///
    pub fn new() -> InMemoryStorage {
        // Create the core
        let core = InMemoryStorageCore {
            animation_properties:   None,
            edit_log:               vec![],
            elements:               HashMap::new(),
            layers:                 HashMap::new(),
            element_attachments:    HashMap::new()
        };

        // And the storage
        InMemoryStorage {
            storage: Arc::new(Desync::new(core))
        }
    }

    ///
    /// Returns the responses for a stream of commands
    ///
    pub fn get_responses<CommandStream: 'static+Send+Unpin+Stream<Item=Vec<StorageCommand>>>(&self, commands: CommandStream) -> impl Send+Unpin+Stream<Item=Vec<StorageResponse>> {
        pipe(Arc::clone(&self.storage), commands, |storage, commands| {
            future::ready(storage.run_commands(commands)).boxed()
        })
    }
}

impl InMemoryStorageCore {
    ///
    /// Runs a series of storage commands on this store
    ///
    pub fn run_commands(&mut self, commands: Vec<StorageCommand>) -> Vec<StorageResponse> {
        let mut response = vec![];

        for command in commands.into_iter() {
            use self::StorageCommand::*;

            match command {
                WriteAnimationProperties(props)                     => { self.animation_properties = Some(props); response.push(StorageResponse::Updated); }
                ReadAnimationProperties                             => { response.push(self.animation_properties.as_ref().map(|props| StorageResponse::AnimationProperties(props.clone())).unwrap_or(StorageResponse::NotFound)); }
                WriteEdit(edit)                                     => { self.edit_log.push(edit); response.push(StorageResponse::Updated); }
                ReadHighestUnusedElementId                          => { response.push(StorageResponse::HighestUnusedElementId(self.elements.keys().cloned().max().unwrap_or(-1)+1)); }
                ReadEditLogLength                                   => { response.push(StorageResponse::NumberOfEdits(self.edit_log.len())); }
                ReadEdits(edit_range)                               => { response.extend(edit_range.into_iter().map(|index| StorageResponse::Edit(index, self.edit_log[index].clone()))); }
                WriteElement(element_id, value)                     => { self.elements.insert(element_id, value); response.push(StorageResponse::Updated); }
                ReadElement(element_id)                             => { response.push(self.elements.get(&element_id).map(|element| StorageResponse::Element(element_id, element.clone())).unwrap_or(StorageResponse::NotFound)); }
                DeleteElement(element_id)                           => { self.elements.remove(&element_id); response.push(StorageResponse::Updated); }
                AddLayer(layer_id, properties)                      => { self.layers.insert(layer_id, InMemoryLayerStorage::new(properties)); response.push(StorageResponse::Updated); }
                
                DeleteLayer(layer_id)                               => { 
                    if self.layers.remove(&layer_id).is_some() { 
                        // TODO: remove element attachments from all of the keyframes
                        response.push(StorageResponse::Updated); 
                    } else { 
                        response.push(StorageResponse::NotFound); 
                    } 
                }

                ReadLayers                                          => { 
                    for (layer_id, storage) in self.layers.iter() {
                        response.push(StorageResponse::LayerProperties(*layer_id, storage.properties.clone()));
                    }
                }
                
                WriteLayerProperties(layer_id, properties)          => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        layer.properties = properties;
                        response.push(StorageResponse::Updated);
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadLayerProperties(layer_id)                       => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        response.push(StorageResponse::LayerProperties(layer_id, layer.properties.clone()));
                    } else {
                        response.push(StorageResponse::NotFound);
                    }
                }

                AddKeyFrame(layer_id, when)                         => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the location where the keyframe can be added
                        match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            Ok(_)           => {
                                // This keyframe already exists
                                response.push(StorageResponse::NotReplacingExisting)
                            }

                            Err(location)   => {
                                // Need to add a new keyframe
                                let keyframe = InMemoryKeyFrameStorage::new(when);
                                layer.keyframes.insert(location, keyframe);

                                response.push(StorageResponse::Updated);
                            }
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                DeleteKeyFrame(layer_id, when)                      => { 
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the location where the keyframe needs to be removed
                        match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            Ok(location)    => {
                                // Exact match of a keyframe
                                //  TODO: remove the attachments for the elements
                                layer.keyframes.remove(location);

                                response.push(StorageResponse::Updated)
                            }

                            Err(_)          => {
                                // No keyframe at this location
                                response.push(StorageResponse::NotFound);
                            }
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                ReadKeyFrames(layer_id, period)                     => {
                    if let Some(layer) = self.layers.get(&layer_id) {
                        // Search for the initial keyframe
                        let initial_keyframe_index = match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&period.start)) {
                            // Period starts at an exact keyframe
                            Ok(location)    => location,

                            // Period covers the keyframe before the specified location if we get a partial match
                            Err(location)   => if location > 0 { location - 1 } else { location }
                        };

                        // Return keyframes until we reach the end of the period
                        let mut keyframe_index = initial_keyframe_index;
                        while keyframe_index < layer.keyframes.len() && layer.keyframes[keyframe_index].when < period.end {
                            // Work out when this keyframe starts and ends
                            let start   = layer.keyframes[keyframe_index].when;
                            let end     = if keyframe_index+1 < layer.keyframes.len() {
                                layer.keyframes[keyframe_index+1].when
                            } else {
                                Duration::new(u64::max_value(), 0)
                            };

                            // Add to the response
                            response.push(StorageResponse::KeyFrame(start, end));

                            // Move on to the next keyframe
                            keyframe_index += 1;
                        }

                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                AttachElementToLayer(layer_id, element_id, when)    => {
                    if let Some(layer) = self.layers.get_mut(&layer_id) {
                        // Search for the keyframe containing this time
                        let keyframe_index = match layer.keyframes.binary_search_by(|frame| frame.when.cmp(&when)) {
                            // Period starts at an exact keyframe
                            Ok(location)    => Some(location),

                            // Period covers the keyframe before the specified location if we get a partial match
                            Err(location)   => if location > 0 { Some(location - 1) } else { None }
                        };

                        if let Some(keyframe_index) = keyframe_index {
                            // Attach to this keyframe
                            layer.keyframes[keyframe_index].attached_elements.insert(element_id, when);

                            self.element_attachments.entry(element_id)
                                .or_insert_with(|| vec![])
                                .push(ElementAttachment {
                                    layer_id:       layer_id, 
                                    keyframe_time:  layer.keyframes[keyframe_index].when
                                });
                        } else {
                            // Keyframe not found
                            response.push(StorageResponse::NotFound);
                        }
                    } else {
                        // Layer not found
                        response.push(StorageResponse::NotFound);
                    }
                }

                DetachElementFromLayer(element_id)                  => { }
                ReadElementAttachments(element_id)                  => { }
                ReadElementsForKeyFrame(layer_id, when)             => { }
            }
        }

        response
    }
}

impl InMemoryLayerStorage {
    ///
    /// Creates a new in-memory layer storage object
    ///
    pub fn new(properties: String) -> InMemoryLayerStorage {
        InMemoryLayerStorage {
            properties: properties,
            keyframes:  vec![]
        }
    }
}

impl InMemoryKeyFrameStorage {
    ///
    /// Creates a new in-memory keyframe storage object
    ///
    pub fn new(when: Duration) -> InMemoryKeyFrameStorage {
        InMemoryKeyFrameStorage {
            when:               when,
            attached_elements:  HashMap::new()
        }
    }
}
