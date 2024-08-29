use bevy_color::{Color, LinearRgba};
use bevy_serialization_extras::prelude::material::MaterialFlag;
use bevy_serialization_extras::prelude::{
    mesh::{GeometryFile, GeometryFlag, MeshPrimitive},
    FileCheckPicker,
};
use bevy_utils::default;
use derive_more::From;
use glam::Vec3;
use nalgebra::Vector3;
use urdf_rs::Visual;
use urdf_rs::Collision;

#[derive(From, Clone)]
pub struct VisualWrapper(Visual);

impl From<&VisualWrapper> for MaterialFlag {
    fn from(value: &VisualWrapper) -> Self {
        if let Some(material) = &value.0.material {
            if let Some(color) = &material.color {
                let rgba = color.rgba.0;
                Self {
                    color: Color::LinearRgba(LinearRgba {
                        red: rgba[0] as f32,
                        green: rgba[1] as f32,
                        blue: rgba[2] as f32,
                        alpha: rgba[3] as f32,
                    }),
                }
            } else {
                Self::default()
            }
        } else {
            Self::default()
        }
    }
}

impl HasGeometry for VisualWrapper {
    fn get_geometry(&self) -> &urdf_rs::Geometry {
        &self.0.geometry
    }
}

impl From<&VisualWrapper> for FileCheckPicker<GeometryFlag, GeometryFile> {
    fn from(value: &VisualWrapper) -> Self {
        to_flag_geometry(value)
    }
}

#[derive(From, Clone)]
pub struct CollisionWrapper(Collision);

impl HasGeometry for CollisionWrapper {
    fn get_geometry(&self) -> &urdf_rs::Geometry {
        &self.0.geometry
    }
}

impl From<&CollisionWrapper> for FileCheckPicker<GeometryFlag, GeometryFile> {
    fn from(value: &CollisionWrapper) -> Self {
        to_flag_geometry(value)
    }
}

trait HasGeometry {
    fn get_geometry(&self) -> &urdf_rs::Geometry;
}

fn to_flag_geometry<T>(wrapper: &T) -> FileCheckPicker<GeometryFlag, GeometryFile>
where
    T: HasGeometry,
{
    let urdf_geometry = wrapper.get_geometry();

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

    let flag_geometry = match urdf_geometry {
        urdf_rs::Geometry::Box { size } => {
            let bevy_size = /*urdf_rotation_flip * */ Vector3::new(size[0], size[1], size[2]);
            FileCheckPicker::PureComponent(GeometryFlag {
                primitive: MeshPrimitive::Cuboid {
                    //size: (*size).map(|f| f as f32),
                    size: [
                        bevy_size[0] as f32,
                        bevy_size[1] as f32,
                        bevy_size[2] as f32,
                    ],
                },
                orientation_matrix: box_allign,
            })
        }
        urdf_rs::Geometry::Cylinder { radius, length } => {
            //let bevy_size = Vector3::new(radius, length, radius);
            FileCheckPicker::PureComponent(GeometryFlag {
                primitive: MeshPrimitive::Cylinder {
                    radius: *radius as f32,
                    length: *length as f32,
                },
                orientation_matrix: cylinder_align,
            })
        }
        urdf_rs::Geometry::Capsule { radius, length } => {
            FileCheckPicker::PureComponent(GeometryFlag {
                primitive: MeshPrimitive::Capsule {
                    radius: *radius as f32,
                    length: *length as f32,
                },
                //FIXME:
                ..default()
            })
        }
        urdf_rs::Geometry::Sphere { radius } => FileCheckPicker::PureComponent(GeometryFlag {
            primitive: MeshPrimitive::Sphere {
                radius: *radius as f32,
            },
            ..default()
        }),
        urdf_rs::Geometry::Mesh { filename, .. } => {
            //let asset_source = AssetSource::Package(filename.clone());

            //let asset_path = parse_urdf_source(asset_source);
            //println!("file name is {:#?}", filename);
            FileCheckPicker::PathComponent(
                //AssetSource::Package(filename.clone());
                GeometryFile {
                    //source: AssetSource::Package(filename.clone()),
                    source: filename.clone(),
                },
            )
        }
    };
    return flag_geometry;
}
