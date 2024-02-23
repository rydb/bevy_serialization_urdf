//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{
    prelude::*, window::PrimaryWindow,
};
use bevy_camera_extras::plugins::DefaultCameraPlugin;
use bevy_egui::EguiContext;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_serialization_urdf::{
    loaders::urdf_loader::Urdf,
    plugin::{AssetSourcesUrdfPlugin, UrdfSerializationPlugin},
    ui::{urdf_widgets_ui, CachedUrdf, DEBUG_FRAME_STYLE},
};
use bevy_ui_extras::systems::visualize_right_sidepanel_for;
use moonshine_save::save::Save;

use bevy_serialization_extras::prelude::{
    link::{JointFlag, StructureFlag},
    rigidbodies::RigidBodyFlag,
    *,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_raycast::DefaultRaycastingPlugin;
use strum_macros::Display;

fn main() {
    App::new()
        .insert_resource(SetSaveFile {
            name: "blue".to_owned(),
        })
        .insert_resource(UrdfHandles::default())
        // asset sources
        .add_plugins(AssetSourcesUrdfPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            exit_condition: bevy::window::ExitCondition::OnPrimaryClosed,
            ..Default::default()
        }))
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
        // Ui
        //.add_systems(Update, selector_raycast)
        .add_systems(Update, physics_utilities_ui)
        .add_systems(Update, rapier_joint_info_ui)
        .add_systems(Update, motor_controller_ui)
        .add_systems(Update, urdf_widgets_ui)
        .add_systems(Update, visualize_right_sidepanel_for::<Name>)
        // Demo systems
        .register_type::<Wheel>()
        .add_systems(Startup, setup)
        .add_systems(Startup, queue_urdf_load_requests)
        .add_systems(Update, control_robot)
        .add_systems(Update, make_robots_selectable)
        .add_systems(Update, bind_left_and_right_wheel)
        .add_systems(Update, freeze_spawned_robots)
        .run();
}

// pub fn fixed_joint_friction_test(
//     mut robots: Query<(Entity, &JointFlag), (With<LinkFlag>, Without<FrictionFlag>)>,
//     mut commands: Commands,

// ) {
//     for (e, joint) in robots.iter() {
//             commands.entity(e)
//             .insert(FrictionFlag {
//                 friction: 0.0,
//                 ..default()
//             });
//     }
// }

#[derive(Component)]
pub struct WasFrozen;

//FIXME: physics bodies fly out of control when spawned, this freezes them for the user to unpause until thats fixed.
pub fn freeze_spawned_robots(
    mut robots: Query<
        (Entity, &mut RigidBodyFlag),
        (With<StructureFlag>, Without<JointFlag>, Without<WasFrozen>),
    >,
    mut commands: Commands,
) {
    for (e, mut body) in robots.iter_mut() {
        *body = RigidBodyFlag::Fixed;
        commands.entity(e).insert(WasFrozen);
    }
}
#[derive(Component, Reflect, Display)]
pub enum Wheel {
    Left,
    Right,
}

// #[derive(Component)]
// pub struct WheelLeft;

// #[derive(Component)]
// pub struct WheelRight;

/// find what is "probably" the left and right wheel, and give them a marker.
pub fn bind_left_and_right_wheel(
    robots: Query<(Entity, &Name), (With<JointFlag>, Without<Wheel>)>,
    mut commands: Commands,
) {
    for (e, name) in robots.iter() {
        let name_str = name.to_string().to_lowercase();

        let split_up = name_str.split("_").collect::<Vec<&str>>();

        if split_up.contains(&Wheel::Left.to_string().to_lowercase().as_str()) {
            commands.entity(e).insert(Wheel::Left);
        }
        if split_up.contains(&Wheel::Right.to_string().to_lowercase().as_str()) {
            commands.entity(e).insert(Wheel::Right);
        }
    }
}

#[derive(Resource, Default)]
pub struct UrdfHandles {
    pub handle_vec: Vec<Handle<Urdf>>,
}

pub fn make_robots_selectable(
    robots: Query<(Entity, &StructureFlag), Without<Selectable>>,
    mut commands: Commands,
) {
    for (e, ..) in robots.iter() {
        commands.entity(e).insert(Selectable);
    }
}

pub fn control_robot(
    mut rigid_body_flag: Query<&mut RigidBodyFlag, (Without<JointFlag>, With<StructureFlag>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut wheels: Query<(&mut JointFlag, &Wheel)>,
) {
    let target_speed = 20.0;

    let leftward_key = KeyCode::ArrowLeft;
    let rightward_key = KeyCode::ArrowRight;
    let forward_key = KeyCode::ArrowUp;
    let backward_key = KeyCode::ArrowDown;

    let freeze_key = KeyCode::KeyP;
    let unfreeze_key = KeyCode::KeyO;

    for mut context in primary_window.iter_mut() {
        egui::Window::new("robot controls")
            .frame(DEBUG_FRAME_STYLE)
            .show(context.get_mut(), |ui| {
                ui.label(format!("Freeze key: {:#?}", freeze_key));
                ui.label(format!("unfreeze key {:#?}", unfreeze_key));
                ui.label("-------------------------");
                ui.label("");
                ui.label("wheel controls")
            });
    }
    for (mut joint, wheel) in wheels.iter_mut() {
        for axis in joint.motors.iter_mut() {
            if keys.pressed(forward_key) {
                axis.target_vel = target_speed
            } else if keys.pressed(backward_key) {
                axis.target_vel = -target_speed
            } else {
                axis.target_vel = 0.0
            }
        }
        match wheel {
            Wheel::Left => {
                for axis in joint.motors.iter_mut() {
                    if keys.pressed(leftward_key) {
                        axis.target_vel = -target_speed
                    }
                    if keys.pressed(rightward_key) {
                        axis.target_vel = target_speed
                    }
                }
            }
            Wheel::Right => {
                for axis in joint.motors.iter_mut() {
                    if keys.pressed(leftward_key) {
                        axis.target_vel = target_speed
                    }
                    if keys.pressed(rightward_key) {
                        axis.target_vel = -target_speed
                    }
                }
            }
        }
    }

    if keys.pressed(freeze_key) {
        for mut rigidbody in rigid_body_flag.iter_mut() {
            *rigidbody = RigidBodyFlag::Fixed;
        }
    }
    if keys.pressed(unfreeze_key) {
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

    urdf_load_requests.requests.push_front(AssetSpawnRequest {
        source: load_urdf_path.to_owned().into(),
        position: Transform::from_xyz(0.0, 2.0, 0.0),
        ..Default::default()
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Plane3d::new(
                    Vec3::new(0.0, 1.0, 0.0)
                ).mesh().size(50.0, 50.0)
            ),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        },
        PhysicsBundle::default(),
    ));

    // light
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
        Save,
    ));
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        Save,
    ));
}

#[derive(Resource, Default)]
pub struct SetSaveFile {
    pub name: String,
}
