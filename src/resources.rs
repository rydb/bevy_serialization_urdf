use bevy_asset::Handle;
use bevy_ecs::prelude::*;

use crate::loaders::urdf_loader::Urdf;

#[derive(Resource, Default)]
pub struct CachedUrdf {
    pub urdf: Handle<Urdf>,
}