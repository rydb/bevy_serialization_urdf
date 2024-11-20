use bevy_asset::{io::{file::FileAssetReader, AssetSource}, AssetApp};
/// plugin that contains everything required for a urdf -> bevy conversion
///
/// NOTE: !!! .dae is not supported! If a .dae support plugin gets added, make an issue, and it can be added.
/// In the meantime, use .obj!!!
///
//FIXME: Add .dae support
// use bevy::{
//     asset::io::{file::FileAssetReader, AssetSource},
//     prelude::*,
// };
use bevy_serialization_extras::prelude::SerializeManyAsOneFor;

use bevy_app::prelude::*;

use crate::{
    loaders::urdf_loader::{Urdf, UrdfLoaderPlugin}, resources::CachedUrdf, wrappers::LinkQuery
};

const PACKAGE: &str = "package";

/// asset sources for urdf. Needs to be loaded before [`DefaultPlugins`]
pub struct AssetSourcesUrdfPlugin {
    // path to folder that `package://`` leads to
    pub assets_folder_local_path: String
}

impl Plugin for AssetSourcesUrdfPlugin {
    fn build(&self, app: &mut App) {
        let path = self.assets_folder_local_path.clone();
        app.register_asset_source(
            PACKAGE,
            AssetSource::build().with_reader(
                move || Box::new(FileAssetReader::new(path.clone()))
            ),
        );
    }
}

pub struct UrdfSerializationPlugin;

impl Plugin for UrdfSerializationPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(UrdfLoaderPlugin)
        .insert_resource(CachedUrdf::default())
        .add_plugins(SerializeManyAsOneFor::<LinkQuery, Urdf>::default());
    }
}
