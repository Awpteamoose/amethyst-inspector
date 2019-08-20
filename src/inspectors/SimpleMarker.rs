use crate::prelude::*;
use amethyst::{
	ecs::saveload::{SimpleMarker, SimpleMarkerAllocator, MarkerAllocator, Marker},
};
use crate::Inspect;

impl<'a> Inspect<'a> for SimpleMarker<()> {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, ..): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			ui.text(im_str!("id: {}", me.id()));
		});
	}

	fn add((_, lazy): &mut Self::SystemData, entity: Entity) {
		lazy.exec(move |w| {
			let (mut marker_s, mut marker_allocator) = w.system_data::<(WriteStorage<'_, SimpleMarker<()>>, Write<'_, SimpleMarkerAllocator<()>>)>();
			marker_allocator.mark(entity, &mut marker_s).unwrap_or_else(f!());
		});
	}
}
