use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui;
use crate::Inspect;

pub type TextureList = Vec<(&'static str, amethyst::renderer::TextureHandle)>;

impl<'a> Inspect<'a> for amethyst::renderer::TextureHandle {
	type SystemData = (
		ReadStorage<'a, Self>,
		Read<'a, TextureList>,
		Read<'a, LazyUpdate>,
	);

	fn inspect((storage, texture_list, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) {
			x
		} else {
			return;
		};
		let mut new_me = me.clone();

		if !texture_list.is_empty() {
			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(texture_list.len());
			for (i, texture) in texture_list.iter().enumerate() {
				if new_me == texture.1 {
					current = i as i32;
				}
				items.push(imgui::im_str!("{}", texture.0).into());
			}

			ui.combo(imgui::im_str!("texture##texture{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me = texture_list[current as usize].1.clone();
		}

		if *me != new_me {
			lazy.insert(entity, new_me);
		}
	}

	fn can_add((_, texture_list, _): &Self::SystemData, _: Entity) -> bool {
		!texture_list.is_empty()
	}

	fn add((_, texture_list, lazy): &Self::SystemData, entity: Entity) {
		// idk if I should insert UiTransform since idk if anything but the ui uses TextureHandle component
		lazy.insert(entity, texture_list[0].1.clone());
	}
}
