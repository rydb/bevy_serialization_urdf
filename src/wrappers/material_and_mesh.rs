use bevy::{prelude::default, render::color::Color};
use bevy_serialization_extras::prelude::{material::MaterialFlag, AssetSource};
use urdf_rs::Visual;
use derive_more::From;
use bevy_serialization_extras::prelude::{FileCheckPicker, mesh::{GeometryFlag, GeometryFile, MeshPrimitive}};
use nalgebra::{Matrix3, Vector3};

use bevy::prelude::Vec3;

#[derive(From, Clone)]
pub struct VisualWrapper(Visual);

impl From<&VisualWrapper> for MaterialFlag {
    fn from(value: &VisualWrapper) -> Self {
        if let Some(material) = &value.0.material {
            if let Some(color) = &material.color {
                let rgba = color.rgba.0;
                Self {
                    color: Color::Rgba { red: rgba[0] as f32, green: rgba[1] as f32, blue: rgba[2] as f32, alpha: rgba[3] as f32 }
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        }
    }
}




impl From<&VisualWrapper> for FileCheckPicker<GeometryFlag, GeometryFile> {

    
    fn from(value: &VisualWrapper) -> Self {
        let visual = &value.0;
        // let urdf_rotation_flipOLD = Matrix3::new(
        //     0.0, 0.0, -1.0,
        //     0.0, 1.0, 0.0,
        //     1.0, 0.0, 0.0,
        // );
        let box_allign = [
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ];
        let cylinder_align = [
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];


            let urdf_geometry = &visual.geometry;

            let flag_geometry = match urdf_geometry {
                urdf_rs::Geometry::Box { size } => { 

                    let bevy_size = /*urdf_rotation_flip * */ Vector3::new(size[0], size[1], size[2]);
                    FileCheckPicker::PureComponent(
                    
                        GeometryFlag { 

                            primitive:  MeshPrimitive::Box {
                                //size: (*size).map(|f| f as f32),
                                size: [bevy_size[0] as f32, bevy_size[1] as f32, bevy_size[2] as f32]
                            },
                            orientation_matrix: box_allign
                        }
                    
                    )
                },
                urdf_rs::Geometry::Cylinder { radius, length } => 
                {
                    //let bevy_size = Vector3::new(radius, length, radius);
                    FileCheckPicker::PureComponent(
                        GeometryFlag {
                            primitive: MeshPrimitive::Cylinder {
                                radius: *radius as f32,
                                length: *length as f32,
                            },
                            orientation_matrix: cylinder_align
                        }
                    )
                },
                urdf_rs::Geometry::Capsule { radius, length } => FileCheckPicker::PureComponent(
                    GeometryFlag {
                        primitive: MeshPrimitive::Capsule {
                            radius: *radius as f32,
                            length: *length as f32,
                        },
                        //FIXME:
                        ..default()
                    }
                ),
                urdf_rs::Geometry::Sphere { radius } => FileCheckPicker::PureComponent(
                    GeometryFlag {
                        primitive: MeshPrimitive::Sphere {
                            radius: *radius as f32,
                        },
                        ..default()
                        
                    }
                ),
                urdf_rs::Geometry::Mesh { filename, .. } => {
                    let asset_source = AssetSource::Package(filename.clone());

                    let asset_path = parse_urdf_source(asset_source);
                    FileCheckPicker::PathComponent(
                    //AssetSource::Package(filename.clone());
                    GeometryFile {
                        //source: AssetSource::Package(filename.clone()),
                        source: asset_path
                    }
                )
                }
            };
            return flag_geometry
    }
}

fn parse_urdf_source(source: AssetSource) -> String {

    match source {
        AssetSource::Package(pkg) => pkg.strip_prefix("package://").unwrap().to_owned(),
        AssetSource::Placeholder(..) => panic!("Asset source not implemented for this asset source.")
    }

}