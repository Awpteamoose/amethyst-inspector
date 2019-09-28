use crate::prelude::*;

#[derive(Default, Clone)]
pub struct TransformInspectorData {
	radians: bool,
}

// TODO: realfield thing
impl<'a> Inspect<'a> for Transform {
	type SystemData = (
		ReadStorage<'a, Self>,
		Read<'a, LazyUpdate>,
		Write<'a, TransformInspectorData>,
	);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy, data): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut new_me = me.clone();
			let mut changed = false;
			let id = ui.push_id(im_str!("Transform"));

			new_me.translation_mut().control().null_to(0.).speed(0.05).label(im_str!("translation")).changed(&mut changed).build();

			if data.radians {
				let mut rotation = new_me.rotation().euler_angles().2;
				rotation.control().null_to(0.).speed(0.25f32.to_radians()).label(im_str!("rotation")).changed(&mut changed).build();
				new_me.set_rotation_2d(rotation);
			} else {
				let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
				if rotation == -180. {
					rotation = 180.;
				}
				rotation.control().null_to(0.).speed(0.25).label(im_str!("rotation")).changed(&mut changed).build();
				new_me.set_rotation_2d(rotation.to_radians());
			}
			ui.same_line(0.);
			ui.checkbox(im_str!("radians"), &mut data.radians);

			new_me.scale_mut().control().null_to(1.).speed(0.01).label(im_str!("scale")).changed(&mut changed).build();

			if changed {
				lazy.insert(entity, new_me);
			}

			id.pop(ui);
		});
	}

	fn add((_storage, lazy, _): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, Self::default());
	}
}
