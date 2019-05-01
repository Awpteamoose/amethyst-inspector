use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui;
use crate::Inspect;

pub type TextureList = std::collections::HashMap<String, amethyst::renderer::TextureHandle>;

impl<'a> Inspect<'a> for amethyst::renderer::TextureHandle {
	type SystemData = (
		ReadStorage<'a, Self>,
		Read<'a, TextureList>,
		Read<'a, LazyUpdate>,
	);

	fn inspect((storage, texture_list, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();

		if !texture_list.is_empty() {
			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(texture_list.len());
			let list_vec = texture_list.iter().collect::<Vec<_>>();
			for (i, (key, texture)) in list_vec.iter().enumerate() {
				if new_me == **texture {
					current = i as i32;
				}
				items.push(imgui::im_str!("{}", key).into());
			}

			ui.combo(imgui::im_str!("texture##texture{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me = list_vec[current as usize].1.clone();
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
		lazy.insert(entity, texture_list.values().nth(0).unwrap_or_else(f!()).clone());
	}
}
