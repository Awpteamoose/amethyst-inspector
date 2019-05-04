use amethyst::{
	core::Named,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

impl<'a> Inspect<'a> for Named {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.push_id(im_str!("named"));
		ui.input_text(im_str!("Entity {}/{}", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();

		let new_name = buf.to_str().to_owned();
		if me.name != new_name {
			lazy.insert(entity, Named::new(new_name));
		}
		ui.pop_id();
	}

	fn add((_storage, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id())));
	}
}
