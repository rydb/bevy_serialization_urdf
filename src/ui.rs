use bevy::{prelude::*, window::PrimaryWindow, reflect::TypeInfo, ecs::schedule::{SystemConfig, SystemConfigs}};
use bevy_egui::EguiContext;
//use bevy_rapier3d::dynamics::ImpulseJoint;
use bevy_serialization_extras::prelude::link::{JointFlag, JointAxesMaskWrapper};
use bitvec::{view::BitView, order::Msb0, field::BitField};
use egui::{epaint::Shadow, text::LayoutJob, Align2, Color32, Frame, InnerResponse, Margin, RichText, Rounding, ScrollArea, Stroke, TextFormat, Ui};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, IntoStaticStr, Display};
use std::collections::HashMap;
use moonshine_save::prelude::Save;
use std::any::TypeId;

use egui_extras::{Column, TableBuilder};

use crate::loaders::urdf_loader::Urdf;


pub const DEBUG_FRAME_STYLE: Frame = Frame {
    inner_margin: Margin {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    },
    outer_margin: Margin {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    },
    rounding: Rounding {
        nw: 0.0,
        ne: 0.0,
        sw: 0.0,
        se: 0.0,
    },
    shadow: Shadow::NONE,
    fill: egui::Color32::from_rgba_premultiplied(15,15,15, 128),
    stroke: Stroke {
        width: 1.0,
        color: Color32::BLACK
    }
};


#[derive(Default, EnumIter, Display)]
pub enum UtilityType {
    #[default]
    UrdfInfo,
}
#[derive(Resource, Default)]
pub struct UtilitySelection {
    pub selected: UtilityType
}
#[derive(Resource, Default)]
pub struct CachedUrdf {
    pub urdf: Handle<Urdf>
}

pub fn urdf_widgets_window(
    mut primary_window: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut utility_selection: ResMut<UtilitySelection>,
    mut asset_server: Res<AssetServer>,
    mut cached_urdf: Res<CachedUrdf>,
    mut urdfs: Res<Assets<Urdf>>,


    mut joint_flags: Query<&mut JointFlag>,

    //rapier_joints: Query<&ImpulseJoint>,

) {
    for mut context in primary_window.iter_mut() { 
        egui::Window::new("debug widget window")
        //.title_bar(false)
        .frame(DEBUG_FRAME_STYLE)
        .show(context.get_mut(), |ui|{
            // lay out the ui widget selection menu
            ui.horizontal(|ui| {
                for utility in UtilityType::iter() {
                    if ui.button(utility.to_string()).clicked() {
                        utility_selection.selected = utility;
                    }
                }
            });

            match utility_selection.selected {
                UtilityType::UrdfInfo => {
                    if let Some(urdf) = urdfs.get(cached_urdf.urdf.clone()) {
                        let urdf_as_string = format!("{:#?}", urdf.robot);
                        
                        if ui.button("Copy to clipboard").clicked() {
                            ui.output_mut(|o| o.copied_text = urdf_as_string.to_string());
                        }
                        ScrollArea::vertical().show(
                            ui, |ui| {

                                let job = LayoutJob::single_section(
                                    urdf_as_string,
                                    TextFormat::default()
                                );
                                ui.label(job);
                            }
                        );
                    }                    
                }
            }
        })

        ;
    }
}
