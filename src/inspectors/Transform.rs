use amethyst::{
	core::transform::Transform,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::{Inspect, InspectControl};
use imgui::im_str;

#[derive(Default, Clone)]
pub struct TransformInspectorData {
	radians: bool,
}

impl<'a> Inspect<'a> for Transform<f32> {
	type SystemData = (
		ReadStorage<'a, Self>,
		Read<'a, LazyUpdate>,
		Write<'a, TransformInspectorData>,
	);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy, data): &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();
		let mut changed = false;
		ui.push_id(im_str!("Transform"));

		changed = new_me.translation_mut().control(0., 0.05, im_str!("translation"), ui) || changed;

		if data.radians {
			let mut rotation = new_me.rotation().euler_angles().2;
			changed = rotation.control(0., 0.25f32.to_radians(), im_str!("rotation"), ui) || changed;
			new_me.set_rotation_2d(rotation);
		} else {
			let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			changed = rotation.control(0., 0.25, im_str!("rotation"), ui) || changed;
			new_me.set_rotation_2d(rotation.to_radians());
		}
		ui.same_line(0.);
		ui.checkbox(im_str!("radians"), &mut data.radians);

		changed = new_me.scale_mut().control(1., 0.01, im_str!("scale"), ui) || changed;

		if changed {
			lazy.insert(entity, new_me);
		}

		ui.pop_id();
	}

	fn add((_storage, lazy, _): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, Transform::<f32>::default());
	}
}
