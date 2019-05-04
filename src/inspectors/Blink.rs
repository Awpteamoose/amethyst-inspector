use amethyst::{
	ecs::prelude::*,
	renderer::Blink,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

impl<'a> Inspect<'a> for Blink {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let &Blink {
			mut delay,
			timer,
			mut absolute_time,
		} = if let Some(x) = storage.get(entity) { x } else { return; };
		ui.push_id(im_str!("blink"));

		let mut changed = ui.drag_float(im_str!("delay"), &mut delay)
			.speed(0.1)
			.build();
		changed = ui.checkbox(im_str!("absolute time"), &mut absolute_time) || changed;

		if changed {
			lazy.insert(
				entity,
				Blink {
					delay,
					timer,
					absolute_time,
				},
			);
		}
		ui.pop_id();
	}

	fn add((_storage, lazy): &mut Self::SystemData, entity: Entity) {
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
