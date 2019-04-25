use amethyst::{
	core::transform::Transform,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;

impl<'a> Inspect<'a> for Transform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();

		{
			let translation = new_me.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			new_me.set_translation(v.into());
		}

		{
			let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			ui.drag_float(imgui::im_str!("rotation##transform{:?}", entity), &mut rotation)
				.speed(0.25)
				.build();
			new_me.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = new_me.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			new_me.set_scale(v[0], v[1], 1.);
		}

		if *me != new_me {
			lazy.insert(entity, new_me);
		}

		// let mut current_entity
		// let parent = parents.get(entitiy);

		// for (i, entity) in entities.join().enumerate() {
		//     if let Some(e) = parent {
		//         if e == entity {
		//             current_entity = i;
		//         }
		//     }
		//     if entity == Some(parent)
		// }
		// ui.combo(imgui::im_str!("parent##transform{:?}", entity), &mut current_item, &items, height_in_items: 10);
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Transform::default());
	}
}
