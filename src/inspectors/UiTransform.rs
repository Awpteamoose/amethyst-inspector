use amethyst::{
	ecs::prelude::*,
	ui::UiTransform,
};
use amethyst_imgui::imgui::{self, im_str};
use crate::Inspect;
use crate::prelude::*;

impl<'a> Inspect<'a> for UiTransform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			use amethyst::ui::ScaleMode;

			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut new_me = me.clone();
			let mut changed = false;
			let id = ui.push_id(im_str!("ui_transform"));

			{
				let mut v: Vector3<f32> = Vector3::new(me.local_x, me.local_y, me.local_z);

				v
					.control()
					.null_to(0.)
					.speed(if me.scale_mode == ScaleMode::Pixel { 1. } else { 0.001 })
					.label(im_str!("translation"))
					.changed(&mut changed)
					.build();

				new_me.local_x = v[0];
				new_me.local_y = v[1];
				new_me.local_z = v[2];
			}

			{
				let mut v: Vector2<f32> = Vector2::new(me.width, me.height);

				v
					.control()
					.null_to(if me.scale_mode == ScaleMode::Pixel { 100. } else { 1. })
					.speed(if me.scale_mode == ScaleMode::Pixel { 1. } else { 0.001 })
					.label(im_str!("size"))
					.changed(&mut changed)
					.build();

				new_me.width = v[0];
				new_me.height = v[1];
			}

			new_me.anchor = inspect_enum!(me.anchor, im_str!("anchor"), [
				amethyst::ui::Anchor::TopLeft,
				amethyst::ui::Anchor::TopMiddle,
				amethyst::ui::Anchor::TopRight,
				amethyst::ui::Anchor::MiddleLeft,
				amethyst::ui::Anchor::Middle,
				amethyst::ui::Anchor::MiddleRight,
				amethyst::ui::Anchor::BottomLeft,
				amethyst::ui::Anchor::BottomMiddle,
				amethyst::ui::Anchor::BottomRight,
			]);

			new_me.pivot = inspect_enum!(me.pivot, im_str!("pivot"), [
				amethyst::ui::Anchor::TopLeft,
				amethyst::ui::Anchor::TopMiddle,
				amethyst::ui::Anchor::TopRight,
				amethyst::ui::Anchor::MiddleLeft,
				amethyst::ui::Anchor::Middle,
				amethyst::ui::Anchor::MiddleRight,
				amethyst::ui::Anchor::BottomLeft,
				amethyst::ui::Anchor::BottomMiddle,
				amethyst::ui::Anchor::BottomRight,
			]);

			{
				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(2);
				let modes = [ScaleMode::Pixel, ScaleMode::Percent];
				for (i, scale_mode) in modes.iter().enumerate() {
					if *scale_mode == me.scale_mode {
						current = i;
					}
					items.push(im_str!("{:?}", scale_mode));
				}

				imgui::ComboBox::new(im_str!("scale mode")).build_simple_string(ui, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice());
				new_me.scale_mode = modes[current as usize].clone();
			}

			id.pop(ui);

			if changed || compare_fields!(me, new_me, anchor, pivot, scale_mode) {
				lazy.insert(entity, new_me);
			}
		});
	}

	fn add((_, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, UiTransform::new(String::default(), amethyst::ui::Anchor::Middle, amethyst::ui::Anchor::Middle, 0., 0., 0., 100., 100.));
	}
}
