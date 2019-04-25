use amethyst::{
	ecs::prelude::*,
	renderer::SpriteRender,
	assets::AssetStorage,
};
use amethyst_imgui::imgui;
use crate::Inspect;

pub type SpriteList = Vec<(&'static str, amethyst::renderer::SpriteSheetHandle)>;

impl<'a> Inspect<'a> for SpriteRender {
	type SystemData = (
		ReadStorage<'a, Self>,
		ReadExpect<'a, AssetStorage<amethyst::renderer::SpriteSheet>>,
		Read<'a, SpriteList>,
		Read<'a, LazyUpdate>,
	);

	fn inspect((storage, sprites, sprite_list, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) {
			x
		} else {
			return;
		};
		let mut new_me = me.clone();

		if !sprite_list.is_empty() {
			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(9);
			for i in 0 .. sprite_list.len() {
				if new_me.sprite_sheet == sprite_list[i].1 {
					current = i as i32;
				}
				items.push(imgui::im_str!("{}", sprite_list[i].0).into());
			}

			ui.combo(imgui::im_str!("sprite sheet##sprite_render{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.sprite_sheet = sprite_list[current as usize].1.clone();
			if new_me.sprite_sheet != me.sprite_sheet {
				new_me.sprite_number = 0;
			}
		}

		let mut sprite_number = new_me.sprite_number as i32;
		ui.slider_int(
			imgui::im_str!("# sprite##sprite_render{:?}", entity),
			&mut sprite_number,
			0,
			sprites.get(&new_me.sprite_sheet).unwrap().sprites.len() as i32 - 1,
		)
		.build();
		new_me.sprite_number = sprite_number as usize;

		if compare_fields!(me, new_me, sprite_number, sprite_sheet) {
			lazy.insert(entity, new_me);
		}
	}

	fn can_add((_, _, sprite_list, _): &Self::SystemData, _: Entity) -> bool {
		!sprite_list.is_empty()
	}

	fn add((_, _, sprite_list, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, sprite_list[0].1.clone());
	}
}
