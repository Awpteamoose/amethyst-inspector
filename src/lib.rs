#![allow(clippy::type_complexity, clippy::float_cmp)]

#[macro_use]
macro_rules! f {
	() => { || panic!("{}:{}", file!(), line!()) };
	(_) => { |_| panic!("{}:{}", file!(), line!()) };
}

pub use amethyst;
use amethyst::ecs::prelude::*;
pub use amethyst_imgui;
use amethyst_imgui::imgui::{self, im_str};
pub use paste;
pub use amethyst_inspector_derive::*;

#[macro_use]
macro_rules! compare_fields {
	($first:expr, $second:expr, $($field:ident),+$(,)*) => (
		$($first.$field != $second.$field ||)+ false
	);
}

mod hierarchy;
mod inspectors;
mod utils;

pub use hierarchy::*;
pub use inspectors::{SpriteRender::SpriteList, TextureHandle::TextureList, UiText::FontList, UiTransformDebug::UiTransformDebug};

/// Implement this on your fields to be able to `#[derive(Inspect)]` on your struct
pub trait InspectControl {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool;
}

/// Draggable float
impl InspectControl for f32 {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let mut changed = ui.drag_float(label, self).speed(speed).build();
		if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			changed = true;
			*self = null_to;
		}

		changed
	}
}

/// Draggable int
impl InspectControl for i32 {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let mut changed = ui.drag_int(label, self).speed(speed).build();
		if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			changed = true;
			*self = null_to as i32;
		}

		changed
	}
}

/// Draggable uint
impl InspectControl for u32 {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let mut v = *self as i32;
		let mut changed = ui.drag_int(label, &mut v).speed(speed).min(0).build();
		if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			changed = true;
			*self = null_to as u32;
		}

		if v < 0 {
			v = 0;
		}
		*self = v as u32;

		changed
	}
}

/// Draggable uint
impl InspectControl for usize {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let mut v = *self as i32;
		let mut changed = ui.drag_int(label, &mut v).speed(speed).min(0).build();
		if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			changed = true;
			*self = null_to as usize;
		}

		if v < 0 {
			v = 0;
		}
		*self = v as usize;

		changed
	}
}

/// Draggable uint as milliseconds
impl InspectControl for std::time::Duration {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let mut v = self.as_millis() as i32;
		let mut changed = ui.drag_int(label, &mut v).speed(speed).min(0).build();
		if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			changed = true;
			v = null_to as i32;
		}
		*self = std::time::Duration::from_millis(v as u64);
		changed
	}
}

fn vec_inspect(size: usize, v: &mut [f32], null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
	let mut changed = false;
	ui.push_id(label);

	let spacing = ui.imgui().style().item_inner_spacing.x;
	let width = ((ui.get_window_size().0 - spacing * ((size - 1) as f32 * 1.5)) * 0.65) / size as f32;

	for i in 0 .. size {
		ui.with_id(i as i32, || {
			ui.with_item_width(width, || {
				changed = ui.drag_float(im_str!(""), &mut v[i as usize]).speed(speed).build() || changed;
				if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
					changed = true;
					v[i as usize] = null_to;
				}
				ui.same_line_spacing(0., spacing);
			});
		});
	}

	ui.text(label);
	ui.pop_id();
	changed
}

/// 2 draggable floats
impl InspectControl for amethyst::core::math::Vector2<f32> {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		vec_inspect(2, self.as_mut_slice(), null_to, speed, label, ui)
	}
}

/// 3 draggable floats
impl InspectControl for amethyst::core::math::Vector3<f32> {
	fn control(&mut self, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		vec_inspect(3, self.as_mut_slice(), null_to, speed, label, ui)
	}
}

/// This holds internal state of inspector
#[derive(Default)]
pub struct InspectorState {
	/// a list of options for the loading dropdown
	pub prefabs: Vec<String>,
	/// if `saveload` feature, is enabled clicking "laod" will add selected prefab here
	pub to_load: Vec<String>,
	/// if `saveload` feature, is enabled clicking "save" will add inspected entity here
	pub to_save: Vec<(Entity, String)>,
	pub selected: Option<Entity>,
	#[doc(hidden)]
	pub save_name: String,
	#[doc(hidden)]
	pub selected_prefab: usize,
}

/// Any component implementing Inspect and included in your `inspect!` will show up in the inspector
/// Whether the component is addable is decided by either `CAN_ADD` or `can_add(...)`
#[allow(unused_variables)]
pub trait Inspect<'a>: Component {
	type SystemData: SystemData<'a>;
	/// Statically decide if this component is addable
	const CAN_ADD: bool = false;
	/// Statically decide if this component is removeable
	const CAN_REMOVE: bool = true;

	/// This method is only ran if the component contains the selected entity
	fn inspect(data: &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {}
	/// Dynamically decide if this component can be added (e.g. because it requires another component)
	fn can_add(data: &mut Self::SystemData, entity: Entity) -> bool { false }
	fn add(data: &mut Self::SystemData, entity: Entity) {}
	/// This method is ran on all entities, even if none are selected
	fn setup(data: &mut Self::SystemData, entity: Option<Entity>) {}
}

macro_rules! inspect_marker {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &mut Self::SystemData, entity: $crate::amethyst::ecs::Entity) { lazy.insert(entity, Self); }
		}
	};
}

#[macro_export]
macro_rules! inspect_default {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &mut Self::SystemData, entity: $crate::amethyst::ecs::Entity) { lazy.insert(entity, Self::default()); }
		}
	};
}

inspect_marker!(amethyst::renderer::Hidden);
inspect_marker!(amethyst::renderer::HiddenPropagate);
inspect_marker!(amethyst::renderer::ScreenSpace);
inspect_marker!(amethyst::renderer::Transparent);

impl<'a> Inspect<'a> for amethyst::renderer::Flipped {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &mut Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		use amethyst::renderer::Flipped;

		let me = if let Some(x) = storage.get(entity) { *x } else { return; };

		let mut current = 0;
		let mut items = Vec::<imgui::ImString>::with_capacity(9);
		let source = [
			Flipped::None,
			Flipped::Horizontal,
			Flipped::Vertical,
			Flipped::Both,
		];
		for (i, &item) in source.iter().enumerate() {
			if item == me {
				current = i as i32;
			}
			items.push(im_str!("{:?}", item).into());
		}

		ui.combo(im_str!("flip"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 4);

		let new_me = source[current as usize];
		if me != new_me {
			lazy.insert(entity, new_me);
		}
	}

	fn add((_, lazy): &mut Self::SystemData, entity: Entity) { lazy.insert(entity, amethyst::renderer::Flipped::None) }
}

#[macro_export]
macro_rules! inspector {
	($($cmp:ident),+$(,)*) => {
		use $crate::amethyst::{
			prelude::*,
			ecs::prelude::*,
		};

		#[derive(Default)]
		#[allow(missing_copy_implementations)]
		pub struct Inspector;
		impl<'s> System<'s> for Inspector {
			type SystemData = (
				Write<'s, $crate::InspectorState>,
				Read<'s, LazyUpdate>,
				Entities<'s>,
				($(ReadStorage<'s, $cmp>,)+),
				($(<$cmp as $crate::Inspect<'s>>::SystemData,)+),
			);

			#[cfg(feature = "saveload")]
			fn setup(&mut self, res: &mut ::amethyst::ecs::Resources) {
				Self::SystemData::setup(res);
				let mut state = res.fetch_mut::<$crate::InspectorState>();
				state.prefabs = ::std::fs::read_dir("assets/prefabs").unwrap().map(|x| x.unwrap().file_name().into_string().unwrap()).collect();
			}

			$crate::paste::item! {
				fn run(&mut self, (mut inspector_state, lazy, entities, ($([<store $cmp>],)+), ($(mut [<data $cmp>],)+)): Self::SystemData) {
					amethyst_imgui::with(move |ui| {
						use ::amethyst_imgui::imgui::{self, im_str};
						use $crate::Inspect;

						ui.window(im_str!("Inspector"))
							.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
							.build(move || {
								$(<$cmp as Inspect>::setup(&mut [<data $cmp>], inspector_state.selected);)+
								if let Some(entity) = inspector_state.selected {
									if entities.is_alive(entity) {
										if ui.small_button(im_str!("make child##inspector{:?}", entity)) {
											lazy.create_entity(&entities)
												.with(amethyst::core::transform::Parent::new(entity))
												.build();
										}
										ui.same_line(0.);
										if ui.small_button(im_str!("remove##inspector{:?}", entity)) {
											lazy.exec_mut(move |w| w.delete_entity(entity).unwrap());
										}

										if ui.collapsing_header(im_str!("add component")).build() {
											let mut hor_pos = 0.;
											$(
												if (<$cmp as Inspect>::CAN_ADD || <$cmp as Inspect>::can_add(&mut [<data $cmp>], entity)) && ![<store $cmp>].contains(entity) {
													if ui.small_button(im_str!("{}", stringify!($cmp))) {
														<$cmp as Inspect>::add(&mut [<data $cmp>], entity);
													}
													hor_pos += ui.get_item_rect_size().0 + ui.imgui().style().item_spacing.x;
													if hor_pos + ui.get_item_rect_size().0 < ui.get_content_region_avail().0 {
														ui.same_line(0.);
													} else {
														hor_pos = 0.;
													}
												}
											)+
											if hor_pos > 0. {
												ui.new_line();
											}

											ui.separator();
										}

										$(
											if [<store $cmp>].contains(entity) {
												let mut remove = false;
												let expanded = ui.collapsing_header(im_str!("{}##header{:?}", stringify!($cmp), entity)).flags(imgui::ImGuiTreeNodeFlags::AllowItemOverlap).default_open(true).build();
												if <$cmp as Inspect>::CAN_REMOVE {
													ui.same_line(0.);
													remove = ui.small_button(im_str!("remove##{}_header_remove", stringify!($cmp)));
												}
												if remove {
													lazy.remove::<$cmp>(entity);
												} else if expanded {
													<$cmp as Inspect>::inspect(&mut [<data $cmp>], entity, ui);
												}
											}
										)+

										#[cfg(feature = "saveload")]
										{
											ui.separator();

											{
												let mut buf = imgui::ImString::new(inspector_state.save_name.clone());
												ui.input_text(im_str!("##inspector_save_input"), &mut buf)
													.resize_buffer(true)
													.build();
												inspector_state.save_name = buf.to_str().to_owned();
											}

											ui.same_line(0.);
											if ui.small_button(im_str!("save##inspector_save_button")) {
												let name = inspector_state.save_name.clone();
												inspector_state.to_save.push((entity, name));
											}
										}
									}
								}

								#[cfg(feature = "saveload")]
								{
									let mut current = inspector_state.selected_prefab as i32;
									let strings = inspector_state.prefabs.iter().map(|x| imgui::ImString::from(im_str!("{}", x))).collect::<Vec<_>>();
									ui.combo(
										im_str!("##inspector_load_combo"),
										&mut current,
										strings.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(),
										10,
									);
									inspector_state.selected_prefab = current as usize;
									ui.same_line(0.);
									if ui.small_button(im_str!("load##inspector_load_button")) {
										let x = inspector_state.prefabs[inspector_state.selected_prefab].clone();
										inspector_state.to_load.push(x);
									}
								}
							});
					});
				}
			}
		}
	};
}
