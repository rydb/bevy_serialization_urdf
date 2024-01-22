
use std::{collections::HashMap, f32::consts::PI};

// use bevy::core::Name;
use bevy::{prelude::*, utils::thiserror, ecs::query::WorldQuery};
//use bevy_rapier3d::geometry::Group;
use bevy_serialization_extras::prelude::{*, link::{LinkFlag, StructureFlag, GeometryShiftMarked, JointFlag, JointAxesMaskWrapper, JointLimitWrapper, Dynamics}, mass::MassFlag, material::MaterialFlag, colliders::ColliderFlag, solvergroupfilter::{SolverGroupsFlag, GroupWrapper}, rigidbodies::RigidBodyFlag, continous_collision::CcdFlag, mesh::{GeometryFlag, GeometryFile}};
use nalgebra::{Matrix3, Vector3};
use urdf_rs::{Robot, Joint, Pose, UrdfError, Link, Visual};

use derive_more::From;

//use crate::{queries::FileCheckPicker, resources::AssetSpawnRequest, loaders::urdf_loader::Urdf, traits::{LazyDeserialize, LoadError}, wrappers::link::LinkFlag};

//use super::{material::MaterialFlag, link::{JointFlag, LinkQuery, JointAxesMaskWrapper, StructureFlag}, mass::MassFlag, colliders::ColliderFlag, rigidbodies::RigidBodyFlag, continous_collision::CcdFlag, solvergroupfilter::SolverGroupsFlag, collisiongroupfilter::CollisionGroupsFlag};
use bevy::render::mesh::VertexAttributeValues::Float32x3;

use crate::loaders::urdf_loader::Urdf;

use super::material_and_mesh::VisualWrapper;


/// the collection of things that qualify as a "link", in the ROS 2 context. 
#[derive(WorldQuery)]
pub struct LinkQuery {
    pub name: Option<&'static Name>,
    pub structure: &'static StructureFlag,
    pub inertial: Option<&'static MassFlag>,
    pub visual: FileCheck<GeometryFlag, GeometryFile>,
    pub collision: Option<&'static ColliderFlag>,
    pub joint: Option<&'static JointFlag>,
}


impl LazyDeserialize for Urdf {
    fn deserialize(absolute_path: String) -> Result<Self, LoadError>{
        let urdf = urdf_rs::read_file(absolute_path)?;
            Ok(Urdf {robot: urdf })
    }
}


pub struct UrdfLinkage<'a, 'b> {
    link: &'a Link,
    joint: Option<&'b Joint>, 
}


impl<'a> FromStructure for Urdf {
    fn into_entities(commands: &mut Commands, value: Self, spawn_request: AssetSpawnRequest<Self>){
        //let name = request.item.clone();
        //let robot = value.world_urdfs.get(&request.item).unwrap();
        //log::info!("urdf is {:#?}", value.clone());

        let robot = value.robot;

        let mut structured_link_map = HashMap::new();
        let mut structured_joint_map = HashMap::new();
        let mut structured_material_map = HashMap::new();

        for joint in &robot.joints {
            structured_joint_map.insert(joint.child.link.clone(), joint.clone());
        }
        for material in &robot.materials {
            structured_material_map.insert(material.name.clone(), material.clone());
        }
        for link in &robot.links {
            structured_link_map.insert(link.name.clone(), link.clone());
        }
        
        // structured_linkage_map.insert(UrdfLinkage {
        //     link:
        // })
        // let query_items =structured_link_map.iter().map(|(key, link)| 
        //     {
        //         LinkQueryItem {
        //             name: Some(&Name::new(link.name.clone())),
        //             structure: &StructureFlag { name: value.name.clone() },
        //             inertial: Some(&MassFlag { mass: link.inertial.mass.value as f32}), 
        //             // implement visual properly
        //             visual: FileCheckItem {component: &GeometryFlag::default(), component_file: None}, 
        //             // implement collision properly. Grouped colliders will need to be ignored for the sake of model coherence.
        //             collision: Some(&ColliderFlag::default()), 
        //             // implement joint loading properly..
        //             joint: Some(&JointFlag::default()) }
        //     }
        // ).collect::<Vec<Self>>();
        let mut structured_entities_map: HashMap<String, Entity> = HashMap::new();


       
        for (key , link) in structured_link_map.iter() {
            let e = *structured_entities_map.entry(link.name.clone())
            .or_insert(commands.spawn_empty().id());


            commands.entity(e)
            .insert(Name::new(link.name.clone()))
            .insert(LinkFlag::from(&link.clone().into()))
            .insert(StructureFlag { name: robot.name.clone() })
            .insert(MassFlag {mass: 1.0})
            //.insert(MassFlag { mass: link.inertial.mass.value as f32})
            ;
            if let Some(visual) = link.visual.first() {
                let visual_wrapper = VisualWrapper::from(visual.clone());
                match FileCheckPicker::from(&visual_wrapper){
                    FileCheckPicker::PureComponent(t) => commands.entity(e).insert(t),
                    FileCheckPicker::PathComponent(u) => commands.entity(e).insert(u),
            };
                commands.entity(e)
                .insert(MaterialFlag::from(&visual_wrapper))
                ;
            } 
            let mut temp_rotate_for_demo = spawn_request.position;
            //FIXME: urdf meshes have their verticies re-oriented to match bevy's cordinate system, but their rotation isn't rotated back
            // to account for this, this will need a proper fix later.
            temp_rotate_for_demo.rotate_x(-PI * 0.5);

            commands.entity(e)
            .insert(VisibilityBundle::default())
            .insert(TransformBundle {
                local: temp_rotate_for_demo, 
                ..default()
            })
            .insert(ColliderFlag::default())
            .insert(SolverGroupsFlag {
                memberships: GroupWrapper::GROUP_1,
                filters: GroupWrapper::GROUP_2,
            })
            .insert(GeometryShiftMarked::default())
            .insert(RigidBodyFlag::Fixed)
            .insert(CcdFlag::default())
            .insert(MatrixVisComponentTest::default())
            //.insert()
            ;
        }

        for (key, joint) in structured_joint_map.iter() {
            let e = *structured_entities_map.entry(joint.child.link.clone())
            .or_insert(commands.spawn_empty().id());

            log::info!("spawning joint on {:#?}", e);
            let new_joint = JointFlag::from(&JointWrapper::from(joint.clone()));

            commands.entity(e)
            .insert(new_joint)
            .insert(RigidBodyFlag::Dynamic)
            ;
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct MatrixVisComponentTest {
    pub matrix: [Vec3; 3]
}

impl IntoHashMap<Query<'_, '_, LinkQuery>> for Urdf {
    fn into_hashmap(value: Query<'_, '_, LinkQuery>) -> HashMap<String, Self> {
        let mut urdf_map = HashMap::new();
        for link in value.iter() {
            let structure_name = link.structure.name.clone();
            let entry = urdf_map.entry(structure_name.clone())
            .or_insert(
                Urdf {
                    robot: Robot { name: link.structure.name.clone(), links: Vec::new(), joints: Vec::new(), materials: Vec::new() }
                }
            )
            ;
            
            match link.joint {
                Some(joint) => {
                    let link_name = link.name.unwrap_or(&Name::new(entry.robot.joints.len().to_string())).to_string();
                    let joint_name = link_name.clone() + "_joint";
                    let joint_parent = joint.parent_name.clone().unwrap_or_default();
                    //let urdf_link_name = link_name + "_link";
                    entry.robot.joints.push
                    (
                        Joint 
                        {
                            name: joint_name,
                            //FIXME:  implement this properly have this be a consequence of joint data via a function. This is a placeholder.
                            joint_type: urdf_rs::JointType::Continuous,
                            origin: Pose {
                                xyz: urdf_rs::Vec3([joint.local_frame1.translation.x.into(), joint.local_frame1.translation.y.into(), joint.local_frame1.translation.z.into()]),
                                rpy: {
                                    let rot = joint.local_frame1.rotation.to_euler(EulerRot::XYZ);
                                    urdf_rs::Vec3([rot.0.into(), rot.1.into(), rot.2.into()])
                                }
                                
                            },
                            parent: urdf_rs::LinkName { link: joint_parent.clone() },
                            child: urdf_rs::LinkName { link: link_name.clone() },
                            axis: urdf_rs::Axis { 
                                xyz:  {
                                    let x = joint.limit_axes.contains(JointAxesMaskWrapper::ANG_X) as u32 as f64;
                                    let y = joint.limit_axes.contains(JointAxesMaskWrapper::ANG_Y) as u32 as f64;
                                    let z = joint.limit_axes.contains(JointAxesMaskWrapper::ANG_Z) as u32 as f64;
                                    urdf_rs::Vec3([x, y, z])
                                }
                            },
                            limit: urdf_rs::JointLimit {
                                lower: joint.limit.lower,
                                upper: joint.limit.upper,
                                //FIXME: implement this properly
                                effort: 99999999999.0,
                                //FIXME: implement this properly
                                velocity: 999999999999.0
                            },
                            //FIXME: implement this properly
                            dynamics: None,
                            //FIXME: implement this properly
                            mimic: None,
                            //FIXME: implement this properly
                            safety_controller: None
                            
                    
                        }
                    )
                }
                None => {}                
            }
        }
        urdf_map
    }
}
#[derive(From)]
pub struct LinkWrapper(Link);

impl From<&LinkWrapper> for LinkFlag {
    fn from(value: &LinkWrapper) -> Self {
        let visual = value.0.visual.first()
        .unwrap_or(&Visual::default())
        .to_owned();
        Self {
            //FIXME: implement this properly to account for urdfs with multiple visual elements 
            geom_offset: Vec3::from_array([visual.origin.xyz[0] as f32, visual.origin.xyz[1] as f32, visual.origin.xyz[2] as f32])
        }
    }
}

#[derive(From)]
pub struct UrdfTransform(Pose);

impl From<UrdfTransform> for Transform {
    fn from(value: UrdfTransform) -> Self {
        // based on this explanation
        //https://towardsdatascience.com/change-of-basis-3909ef4bed43
        let urdf_cord_flip = Matrix3::new(
            1.0, 0.0, 0.0,
            0.0, 0.0, 1.0,
            0.0, 1.0, 0.0,
        );
        // based on this explanation
        //https://stackoverflow.com/questions/31191752/right-handed-euler-angles-xyz-to-left-handed-euler-angles-xyz
        let urdf_rotation_flip = Matrix3::new(
            -1.0, 0.0, 0.0,
            0.0, -1.0, 0.0,
            0.0, 0.0, 1.0,
        );
        let pos = value.0;
        
        let compliant_trans = 
            /*urdf_cord_flip * */Vector3::new(pos.xyz.0[0], pos.xyz.0[1], pos.xyz.0[2]);
        let compliant_rot = 
            /*urdf_rotation_flip * */Vector3::new(pos.rpy.0[0], pos.rpy.0[1], pos.rpy.0[2]);


        Self {
            translation:  Vec3::new(compliant_trans.x as f32, compliant_trans.y as f32, compliant_trans.z as f32),
            rotation: Quat::from_euler(EulerRot::XYZ, compliant_rot.x as f32, compliant_rot.y as f32, compliant_rot.z as f32),
            ..default()
        }
    }
} 

#[derive(From)]
pub struct JointWrapper(Joint);

impl From<&JointWrapper> for JointFlag {
    fn from(value: &JointWrapper) -> Self {
        let joint_offset = Transform::from(UrdfTransform::from(value.0.origin.clone()));
        Self {
            offset: Transform {
                 translation: Vec3::new(value.0.origin.xyz.0[0] as f32, value.0.origin.xyz.0[1] as f32, value.0.origin.xyz.0[2] as f32),
                // rotation: Quat::default(),
                ..default()
            },
            parent_name: Some(value.0.parent.link.clone()),
            parent_id: None,
            limit: JointLimitWrapper {
                 lower:  value.0.limit.lower, 
                 upper: value.0.limit.upper, 
                 effort: value.0.limit.effort, 
                 velocity: value.0.limit.velocity
            },
            dynamics: {
                match value.0.dynamics.clone() {
                    Some(dynamics) => 
                        Dynamics {
                            damping: dynamics.damping,
                            friction: dynamics.friction,
                        },
                    None => Dynamics::default()
                    
                }
            },
            local_frame1: UrdfTransform::from(value.0.origin.clone()).into(),
            local_frame2: None,
            locked_axes: {
                //clamp axis to between 0-1 for simplicity and for bitmask flipping
                // let unit_axis = value.0.axis.xyz.0
                // .map(|n| n.clamp(0.0, 1.0))
                // .map(|n| n as u8);
                // let mut x = 1 << unit_axis[0];
                // x = x | (2 << unit_axis[1]);
                // x = x | (3 << unit_axis[2]);
                // JointAxesMaskWrapper::from_bits_truncate(x)
                JointAxesMaskWrapper::LOCKED_FIXED_AXES
            },
            limit_axes: JointAxesMaskWrapper::empty(),
            motor_axes: JointAxesMaskWrapper::empty(),
            coupled_axes: JointAxesMaskWrapper::empty(),
            contacts_enabled: true,
            enabled: true
        }
    }
}