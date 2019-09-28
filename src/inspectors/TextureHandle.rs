use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

/// Add this as a resource and insert your handles into it to get a dropdown for TextureHandle selection
pub type TextureList = std::collections::HashMap<String, amethyst::renderer::TextureHandle>;

impl<'a> Inspect<'a> for amethyst::renderer::TextureHandle {
	type SystemData = (
		ReadStorage<'a, Self>,
		Read<'a, TextureList>,
		Read<'a, LazyUpdate>,
	);

	fn inspect((storage, texture_list, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut new_me = me.clone();
			let id = ui.push_id(im_str!("texture"));

			if !texture_list.is_empty() {
				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(texture_list.len());
				let list_vec = texture_list.iter().collect::<Vec<_>>();
				for (i, (key, texture)) in list_vec.iter().enumerate() {
					if new_me == **texture {
						current = i as i32;
					}
					items.push(im_str!("{}", key).into());
				}

				ui.combo(im_str!("texture"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
				new_me = list_vec[current as usize].1.clone();
			}

			if *me != new_me {
				lazy.insert(entity, new_me);
			}
			id.pop(ui);
		});
	}

	fn can_add((_, texture_list, _): &mut Self::SystemData, _: Entity) -> bool {
		!texture_list.is_empty()
	}

	fn add((_, texture_list, lazy): &mut Self::SystemData, entity: Entity) {
		// idk if I should insert UiTransform since idk if anything but the ui uses TextureHandle component
		lazy.insert(entity, texture_list.values().nth(0).unwrap_or_else(f!()).clone());
	}
}
