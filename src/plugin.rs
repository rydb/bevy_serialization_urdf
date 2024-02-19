/// plugin that contains everything required for a urdf -> bevy conversion
///
/// NOTE: !!! .dae is not supported! If a .dae support plugin gets added, make an issue, and it can be added.
/// In the meantime, use .obj!!!
///
//FIXME: Add .dae support
use bevy::{
    asset::io::{file::FileAssetReader, AssetSource},
    prelude::*,
};
use bevy_serialization_extras::prelude::SerializeManyAsOneFor;

use crate::{
    loaders::urdf_loader::{Urdf, UrdfLoaderPlugin},
    ui::{CachedUrdf, UtilitySelection},
    wrappers::LinkQuery,
};

const PACKAGE: &str = "package";

/// asset sources for urdf. Needs to be loaded before [`DefaultPlugins`]
pub struct AssetSourcesUrdfPlugin;

impl Plugin for AssetSourcesUrdfPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            PACKAGE,
            AssetSource::build().with_reader(|| Box::new(FileAssetReader::new("../../assets"))),
        );
    }
}

pub struct UrdfSerializationPlugin;

impl Plugin for UrdfSerializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UrdfLoaderPlugin)
            .insert_resource(UtilitySelection::default())
            .insert_resource(CachedUrdf::default())
            .add_plugins(SerializeManyAsOneFor::<LinkQuery, Urdf>::default());
    }
}
