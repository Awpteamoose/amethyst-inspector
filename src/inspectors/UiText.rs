use amethyst::{
	ecs::prelude::*,
	renderer::Rgba,
	assets::AssetStorage,
	ui::UiTransform,
};
use amethyst_imgui::imgui;
use crate::Inspect;

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

	const CAN_ADD: bool = true;

	fn inspect((storage, _, _, _, font_list, lazy): &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();

		{
			let mut buf = imgui::ImString::new(me.text.clone());
			ui.input_text(imgui::im_str!("text##ui_text{:?}", entity), &mut buf)
				.resize_buffer(true)
				.build();
			new_me.text = buf.to_str().to_owned();
		}


		if !font_list.is_empty() {
			let mut current = 0;
			let mut items = Vec::<imgui::ImString>::with_capacity(9);
			let list_vec = font_list.iter().collect::<Vec<_>>();
			for (i, (key, font)) in list_vec.iter().enumerate() {
				if me.font == **font {
					current = i as i32;
				}
				items.push(imgui::im_str!("{}", key).into());
			}

			ui.combo(imgui::im_str!("font##ui_text{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.font = list_vec[current as usize].1.clone();
		}

		ui.drag_float(imgui::im_str!("font size##ui_text{:?}", entity), &mut new_me.font_size)
			.speed(0.5)
			.build();

		ui.drag_float4(imgui::im_str!("colour##ui_text{:?}", entity), &mut new_me.color)
			.speed(0.005)
			.build();

		ui.checkbox(imgui::im_str!("password##ui_text{:?}", entity), &mut new_me.password);

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
					current = i as i32;
				}
				items.push(imgui::im_str!("{:?}", line_mode).into());
			}

			ui.combo(imgui::im_str!("line style##ui_text{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
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
					current = i as i32;
				}
				items.push(imgui::im_str!("{:?}", anchor).into());
			}

			ui.combo(imgui::im_str!("align##ui_text{:?}", entity), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
			new_me.align = anchors[current as usize].clone();
		}

		if compare_fields!(me, new_me, text, font_size, color, password, line_mode, align) {
			lazy.insert(entity, new_me);
		}
	}

	fn add((_storage, transforms, loader, fonts, font_list, lazy): &mut Self::SystemData, entity: Entity) {
		let font = if font_list.is_empty() { amethyst::ui::get_default_font(&loader, &fonts) } else { font_list.values().nth(0).unwrap_or_else(f!()).clone() };
		if !transforms.contains(entity) {
			lazy.insert(entity, UiTransform::new(String::default(), amethyst::ui::Anchor::Middle, 0., 0., 0., 100., 100.));
		}
		lazy.insert(entity, amethyst::ui::UiText::new(
			font,
			"Sample text".to_string(),
			Rgba::white().into(),
			30.,
		));
	}
}
