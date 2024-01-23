/// plugin that contains everything required for a urdf -> bevy conversion
/// 
/// NOTE: !!! .dae is not supported! If a .dae support plugin gets added, make an issue, and it can be added.
/// In the meantime, use .obj!!!
/// 
//FIXME: Add .dae support
use bevy::prelude::*;
use bevy_serialization_extras::prelude::SerializeManyAsOneFor;

use crate::{loaders::urdf_loader::{UrdfLoaderPlugin, Urdf}, ui::{UtilitySelection, CachedUrdf}, wrappers::{LinkQuery}};

pub struct UrdfSerializationPlugin;

impl Plugin for UrdfSerializationPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(UrdfLoaderPlugin)

        .insert_resource(UtilitySelection::default())
        .insert_resource(CachedUrdf::default())


        .add_plugins(SerializeManyAsOneFor::<LinkQuery, Urdf>::default())
        ;

    }
}