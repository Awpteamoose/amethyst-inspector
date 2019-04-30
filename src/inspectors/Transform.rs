use amethyst::{
	core::transform::Transform,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

impl<'a> Inspect<'a> for Transform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();
		ui.push_id(im_str!("Ttransform##{:?}", entity));

		{
			let translation = new_me.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			crate::utils::nullable_float3(0., 0.05, im_str!("translation"), &mut v, ui);
			new_me.set_translation(v.into());
		}

		{
			let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			crate::utils::nullable_float(0., 0.25, im_str!("rotation"), &mut rotation, ui);
			new_me.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = new_me.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			crate::utils::nullable_float2(1., 0.01, im_str!("scale"), &mut v, ui);
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
		// ui.combo(im_str!("parent##transform{:?}", entity), &mut current_item, &items, height_in_items: 10);

		ui.pop_id();
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Transform::default());
	}
}
