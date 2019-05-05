use amethyst::{
	ecs::prelude::*,
	renderer::Rgba,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

impl<'a> Inspect<'a> for Rgba {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			ui.push_id(im_str!("rgba"));

			let mut v: [f32; 4] = [me.0, me.1, me.2, me.3];
			ui.drag_float4(im_str!("colour tint"), &mut v)
				.speed(0.005)
				.build();

			if *me != Rgba::from(v) {
				lazy.insert(entity, Rgba::from(v));
			}
			ui.pop_id();
		});
	}

	fn add((_storage, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, amethyst::renderer::Rgba::white());
	}
}
