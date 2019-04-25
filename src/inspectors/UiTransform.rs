use amethyst::{
	ecs::prelude::*,
	ui::UiTransform,
};
use amethyst_imgui::imgui;
use crate::Inspect;

impl<'a> Inspect<'a> for UiTransform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();

		{
			let mut v: [f32; 3] = [me.local_x, me.local_y, me.local_z];
			ui.drag_float3(imgui::im_str!("translation##ui_transform{:?}", entity), &mut v)
				.build();
			new_me.local_x = v[0];
			new_me.local_y = v[1];
			new_me.local_z = v[2];
		}

		// {
		//     let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
		//     if rotation == -180. {
		//         rotation = 180.;
		//     }
		//     ui.drag_float(imgui::im_str!("rotation##transform{:?}", entity), &mut rotation)
		//         .speed(0.25)
		//         .build();
		//     new_me.set_rotation_2d(rotation.to_radians());
		// }

		{
			let mut v: [f32; 2] = [me.width, me.height];
			ui.drag_float2(imgui::im_str!("size##ui_transform{:?}", entity), &mut v)
				.build();
			new_me.width = v[0];
			new_me.height = v[1];
		}

		{
			use amethyst::ui::Anchor;

			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(9);
			let anchors = [
				Anchor::TopLeft,
				Anchor::TopMiddle,
				Anchor::TopRight,
				Anchor::MiddleLeft,
				Anchor::Middle,
				Anchor::MiddleRight,
				Anchor::BottomLeft,
				Anchor::BottomMiddle,
				Anchor::BottomRight,
			];
			for (i, anchor) in anchors.iter().enumerate() {
				if *anchor == me.anchor {
					current = i as i32;
				}
				items.push(imgui::im_str!("{:?}", anchor).into());
			}

			ui.combo(imgui::im_str!("anchor##ui_transform{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.anchor = anchors[current as usize].clone();
		}

		if compare_fields!(me, new_me, local_x, local_y, local_z, width, height, anchor) {
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

	fn add((_, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, UiTransform::new(String::default(), amethyst::ui::Anchor::Middle, 0., 0., 0., 100., 100.));
	}
}
