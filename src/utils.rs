use amethyst::{
	core::transform::Transform,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

pub fn nullable_float(null_to: f32, speed: f32, label: &imgui::ImStr, v: &mut f32, ui: &imgui::Ui<'_>) {
	let spacing = ui.imgui().style().item_inner_spacing.x;
	let width = (ui.get_window_size().0 - spacing) * 0.65;
	ui.with_id(label, || {
		ui.with_item_width(width, || {
			ui.drag_float(im_str!(""), v).speed(speed).build();
			if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
				*v = null_to;
			}
			ui.same_line_spacing(0., spacing);
		});
		ui.text(label);
	});
}

pub fn nullable_float2(null_to: f32, speed: f32, label: &imgui::ImStr, v: &mut [f32; 2], ui: &imgui::Ui<'_>) {
	let spacing = ui.imgui().style().item_inner_spacing.x;
	let width = ((ui.get_window_size().0 - spacing) * 0.65) / 2.;
	ui.with_id(label, || {
		for i in 0 .. 2 {
			ui.with_id(i, || {
				ui.with_item_width(width, || {
					ui.drag_float(im_str!(""), &mut v[i as usize]).speed(speed).build();
					if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
						v[i as usize] = null_to;
					}
					ui.same_line_spacing(0., spacing);
				});
			});
		}
		ui.text(label);
	});
}

pub fn nullable_float3(null_to: f32, speed: f32, label: &imgui::ImStr, v: &mut [f32; 3], ui: &imgui::Ui<'_>) {
	let spacing = ui.imgui().style().item_inner_spacing.x;
	let width = ((ui.get_window_size().0 - spacing * 2.) * 0.65) / 3.;
	ui.with_id(label, || {
		for i in 0 .. 3 {
			ui.with_id(i, || {
				ui.with_item_width(width, || {
					ui.drag_float(im_str!(""), &mut v[i as usize]).speed(speed).build();
					if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
						v[i as usize] = null_to;
					}
					ui.same_line_spacing(0., spacing);
				});
			});
		}
		ui.text(label);
	});
}
