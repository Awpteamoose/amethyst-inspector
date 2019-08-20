use amethyst::{
	ecs::prelude::*,
	renderer::{palette::Srgba, resources::Tint},
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;
use crate::prelude::*;

impl<'a> Inspect<'a> for Tint {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			ui.push_id(im_str!("tint"));

			let (r, g, b, a) = me.0.into_components();
			let mut v: Vector4<f32> = Vector4::new(r, g, b, a);
			let mut changed = false;
			v.control().null_to(1.).speed(0.005).label(im_str!("colour")).changed(&mut changed).build();

			if changed {
				lazy.insert(entity, Tint(Srgba::from_components((v[0], v[1], v[2], v[3]))));
			}
			ui.pop_id();
		});
	}

	fn add((_storage, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, Tint(Srgba::from_components((1., 1., 1., 1.))));
	}
}
