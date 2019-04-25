use amethyst::{
	core::Named,
	ecs::prelude::*,
};
use amethyst_imgui::imgui;
use crate::Inspect;

impl<'a> Inspect<'a> for Named {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.input_text(imgui::im_str!("Entity {}/{}##named", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();

		let new_name = buf.to_str().to_owned();
		if me.name != new_name {
			lazy.insert(entity, Named::new(new_name));
		}
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id())));
	}
}
