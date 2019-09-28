use amethyst::{
	ecs::prelude::*,
	assets::AssetStorage,
	ui::UiTransform,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

/// Add this as a resource and insert your handles into it to get a dropdown for FontHandle selection
pub type FontList = std::collections::HashMap<String, amethyst::ui::FontHandle>;

impl<'a> Inspect<'a> for amethyst::ui::UiText {
	type SystemData = (
		ReadStorage<'a, Self>,
		ReadStorage<'a, UiTransform>,
		ReadExpect<'a, amethyst::assets::Loader>,
		ReadExpect<'a, AssetStorage<amethyst::ui::FontAsset>>,
		Read<'a, FontList>,
		Read<'a, LazyUpdate>,
	);

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn inspect((storage, _, _, _, font_list, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut new_me = me.clone();
			let id = ui.push_id(im_str!("ui_text"));
			let mut changed = false;

			{
				let mut buf = imgui::ImString::new(me.text.clone());
				changed = ui.input_text(im_str!("text"), &mut buf)
					.resize_buffer(true)
					.build() || changed;
				new_me.text = buf.to_str().to_owned();
			}


			if !font_list.is_empty() {
				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(9);
				let list_vec = font_list.iter().collect::<Vec<_>>();
				for (i, (key, font)) in list_vec.iter().enumerate() {
					if me.font == **font {
						current = i;
					}
					items.push(im_str!("{}", key));
				}

				imgui::ComboBox::new(im_str!("font")).build_simple_string(ui, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice());
				new_me.font = list_vec[current as usize].1.clone();
			}

			changed = ui.drag_float(im_str!("font size"), &mut new_me.font_size)
				.speed(0.5)
				.build() || changed;

			changed = ui.drag_float4(im_str!("colour"), &mut new_me.color)
				.speed(0.005)
				.build() || changed;

			changed = ui.checkbox(im_str!("password"), &mut new_me.password) || changed;

			{
				use amethyst::ui::LineMode;

				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(9);
				let line_modes = [
					LineMode::Single,
					LineMode::Wrap,
				];
				for (i, line_mode) in line_modes.iter().enumerate() {
					if *line_mode == me.line_mode {
						current = i;
					}
					items.push(im_str!("{:?}", line_mode));
				}

				changed = imgui::ComboBox::new(im_str!("line style")).build_simple_string(ui, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice()) || changed;
				new_me.line_mode = line_modes[current as usize].clone();
			}

			{
				use amethyst::ui::Anchor;

				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(9);
				let anchors = [
					Anchor::TopLeft,
					Anchor::TopMiddle,
					Anchor::TopRight,
					Anchor::MiddleLeft,
					Anchor::Middle,
					Anchor::MiddleRight,
					Anchor::BottomLeft,
					Anchor::BottomMiddle,
					Anchor::BottomRight,
				];
				for (i, anchor) in anchors.iter().enumerate() {
					if *anchor == me.align {
						current = i;
					}
					items.push(im_str!("{:?}", anchor));
				}

				changed = imgui::ComboBox::new(im_str!("align")).build_simple_string(ui, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice()) || changed;
				new_me.align = anchors[current as usize].clone();
			}

			if changed {
				lazy.insert(entity, new_me);
			}

			id.pop(ui);
		});
	}

	fn add((_storage, transforms, loader, fonts, font_list, lazy): &mut Self::SystemData, entity: Entity) {
		let font = if font_list.is_empty() { amethyst::ui::get_default_font(&loader, &fonts) } else { font_list.values().nth(0).unwrap_or_else(f!()).clone() };
		if !transforms.contains(entity) {
			lazy.insert(entity, UiTransform::new(String::default(), amethyst::ui::Anchor::Middle, amethyst::ui::Anchor::Middle, 0., 0., 0., 100., 100.));
		}
		lazy.insert(entity, amethyst::ui::UiText::new(
			font,
			"Sample text".to_string(),
			[1., 1., 1., 1.],
			30.,
		));
	}
}
