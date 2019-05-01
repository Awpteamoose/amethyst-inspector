use amethyst::{
	ecs::prelude::*,
	ui::UiTransform,
};
use amethyst_imgui::imgui::{self, im_str};
use crate::Inspect;

impl<'a> Inspect<'a> for UiTransform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		use amethyst::ui::ScaleMode;

		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();
		let mut changed = false;
		ui.push_id(im_str!("UiTransform##{:?}", entity));

		{
			let mut v: [f32; 3] = [me.local_x, me.local_y, me.local_z];
			changed = crate::utils::nullable_float3(0., if me.scale_mode == ScaleMode::Pixel { 1. } else { 0.001 }, im_str!("translation"), &mut v, ui) || changed;
			new_me.local_x = v[0];
			new_me.local_y = v[1];
			new_me.local_z = v[2];
		}

		{
			let mut v: [f32; 2] = [me.width, me.height];
			changed = crate::utils::nullable_float2(if me.scale_mode == ScaleMode::Pixel { 100. } else { 0.1 }, if me.scale_mode == ScaleMode::Pixel { 1. } else { 0.001 }, im_str!("size"), &mut v, ui) || changed;
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
				items.push(im_str!("{:?}", anchor).into());
			}

			ui.combo(im_str!("anchor"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.anchor = anchors[current as usize].clone();
		}

		{
			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(2);
			let modes = [ScaleMode::Pixel, ScaleMode::Percent];
			for (i, scale_mode) in modes.iter().enumerate() {
				if *scale_mode == me.scale_mode {
					current = i as i32;
				}
				items.push(im_str!("{:?}", scale_mode).into());
			}

			ui.combo(im_str!("scale mode"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.scale_mode = modes[current as usize].clone();
		}

		if changed || compare_fields!(me, new_me, anchor, scale_mode) {
			lazy.insert(entity, new_me);
		}

		ui.pop_id();
	}

	fn add((_, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, UiTransform::new(String::default(), amethyst::ui::Anchor::Middle, 0., 0., 0., 100., 100.));
	}
}
