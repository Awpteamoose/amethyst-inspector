use crate::prelude::*;
use amethyst::{
	ecs::saveload::{U64Marker, U64MarkerAllocator, MarkerAllocator},
};
use crate::Inspect;

impl<'a> Inspect<'a> for U64Marker {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			ui.text(im_str!("{:?}", me));
		});
	}

	fn add((_, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.exec(move |w| {
			let (mut marker_s, mut marker_allocator) = w.system_data::<(WriteStorage<'_, U64Marker>, Write<'_, U64MarkerAllocator>)>();
			marker_allocator.mark(entity, &mut marker_s).unwrap_or_else(f!()).0;
		});
	}
}
