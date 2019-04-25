use amethyst::{
	ecs::prelude::*,
	renderer::Blink,
};
use amethyst_imgui::imgui;
use crate::Inspect;

impl<'a> Inspect<'a> for Blink {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let &Blink {
			mut delay,
			timer,
			mut absolute_time,
		} = if let Some(x) = storage.get(entity) {
			x
		} else {
			return;
		};
		ui.drag_float(imgui::im_str!("delay##blink{:?}", entity), &mut delay)
			.speed(0.1)
			.build();
		ui.checkbox(imgui::im_str!("absolute time##blink{:?}", entity), &mut absolute_time);

		lazy.insert(
			entity,
			Blink {
				delay,
				timer,
				absolute_time,
			},
		);
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(
			entity,
			Blink {
				delay: 0.5,
				timer: 0.,
				absolute_time: false,
			},
		);
	}
}
