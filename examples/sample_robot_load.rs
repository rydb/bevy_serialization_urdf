//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_serialization_urdf::{plugin::UrdfSerializationPlugin, ui::{CachedUrdf, urdf_widgets_window}, loaders::urdf_loader::Urdf};
use moonshine_save::save::Save;
use bevy_rapier3d::{plugin::{RapierPhysicsPlugin, NoUserData}, render::RapierDebugRenderPlugin};
use bevy_camera_extras::plugins::DefaultCameraPlugin;

use bevy_serialization_extras::prelude::{link::{JointFlag, LinkFlag}, rigidbodies::RigidBodyFlag, *};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_raycast::DefaultRaycastingPlugin;
fn main() {

    App::new()

    .insert_resource(SetSaveFile{name: "blue".to_owned()})
    .insert_resource(UrdfHandles::default())
    .add_plugins(DefaultPlugins.set(WindowPlugin {exit_condition: bevy::window::ExitCondition::OnPrimaryClosed, ..Default::default()}))
        
        // serialization plugins
        .add_plugins(SerializationPlugin)
        .add_plugins(PhysicsSerializationPlugin)
        .add_plugins(UrdfSerializationPlugin)

        // rapier physics plugins
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())


        .add_plugins(DefaultRaycastingPlugin)        
        
        // Quality of life for demo
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(DefaultCameraPlugin)
        .init_resource::<SelectedMotorAxis>()
        .init_resource::<PhysicsUtilitySelection>()


        .add_systems(Startup, queue_urdf_load_requests)
        .add_systems(Startup, setup)
        .add_systems(Update, urdf_widgets_window)
        .add_systems(Update, pause_unpause_bodies)
        .add_systems(Update, selector_raycast)
        .add_systems(Update, motor_controller_ui)
        .add_systems(Update, make_robots_selectable)
        .add_systems(Update, physics_utilities_ui)
        .add_systems(Update, rapier_joint_info_ui)
        .run();
}




#[derive(Resource, Default)]
pub struct UrdfHandles {
    pub handle_vec: Vec<Handle<Urdf>>,

}

pub fn make_robots_selectable(
    robots: Query<(Entity, &LinkFlag), Without<Selectable>>,
    mut commands: Commands,
) {
    for (e ,link) in robots.iter() {
        commands.entity(e)
        .insert(Selectable)
        ;
    }
}

pub fn pause_unpause_bodies(
    mut rigid_body_flag: Query<(&mut RigidBodyFlag), (Without<JointFlag>, With<LinkFlag>)>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.pressed(KeyCode::P) {
        for mut rigidbody in rigid_body_flag.iter_mut() {
            *rigidbody = RigidBodyFlag::Fixed;
        }
    }
    if keys.pressed(KeyCode::O) {
        for mut rigidbody in rigid_body_flag.iter_mut() {
            *rigidbody = RigidBodyFlag::Dynamic;
        }
    }
}

pub fn queue_urdf_load_requests(
    mut urdf_load_requests: ResMut<AssetSpawnRequestQueue<Urdf>>,
    mut cached_urdf: ResMut<CachedUrdf>,
    asset_server: Res<AssetServer>,

) {
    // set load_urdf_path to the urdf you want to load. 

    let load_urdf_path = "model_pkg/urdf/diff_bot.xml";
    //let load_urdf_path = "urdf_tutorial/urdfs/tutorial_bot.xml";
    //let load_urdf_path = "urdf_tutorial/urdfs/issue_test.xml";
    //let load_urdf_path = "urdf_tutorial/urdfs/full_urdf_tutorial_bot.xml";
    
    cached_urdf.urdf = asset_server.load(load_urdf_path);

    urdf_load_requests.requests.push_front(
        AssetSpawnRequest {
             source: load_urdf_path.to_owned().into(), 
             position: Transform::from_xyz(0.0, 1.0, 0.0), 
             ..Default::default()
        }
    );
    
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // plane
    commands.spawn(
    (
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        },
        PhysicsBundle::default()
        )
    );

    // light
    commands.spawn(
        (
        PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    },
    Save
    )
);
    // camera
    commands.spawn(
    (Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },
    Save
));
}


#[derive(Resource, Default)]
pub struct SetSaveFile {
    pub name: String,
}
