use amethyst::{
	core::Named,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

impl<'a> Inspect<'a> for Named {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut buf = imgui::ImString::new(me.name.clone());
			let id = ui.push_id(im_str!("named"));
			ui.input_text(&im_str!("Entity {}/{}", entity.id(), entity.gen().id()), &mut buf)
				.resize_buffer(true)
				.build();

			let new_name = buf.to_str().to_owned();
			if me.name != new_name {
				lazy.insert(entity, Named::new(new_name));
			}
			id.pop(ui);
		});
	}

	fn add((_, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id())));
	}
}
