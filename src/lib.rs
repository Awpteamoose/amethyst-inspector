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
pub mod utils;

pub use hierarchy::*;
pub use inspectors::{SpriteRender::SpriteList, TextureHandle::TextureList, UiText::FontList, UiTransformDebug::*};

pub trait InspectControl {
	fn control(&mut self, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool;
}

impl InspectControl for f32 {
	fn control(&mut self, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		ui.drag_float(label, self).build()
	}
}

impl InspectControl for amethyst::core::math::Vector2<f32> {
	fn control(&mut self, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
		let null_to = 0.;
		let speed = 1.;

		let mut v: [f32; 2] = [self[0], self[1]];
		let mut changed = false;

		let spacing = ui.imgui().style().item_inner_spacing.x;
		let width = ((ui.get_window_size().0 - spacing) * 0.65) / 2.;

		for i in 0 .. 2 {
			ui.with_id(i, || {
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
		self[0] = v[0];
		self[1] = v[1];

		changed
	}
}

#[derive(Default)]
pub struct InspectorState {
	pub selected_prefab: usize,
	pub prefabs: Vec<String>,
	pub to_load: Vec<String>,
	pub to_save: Vec<(Entity, String)>,
	pub selected: Option<Entity>,
	pub save_name: String,
}

#[allow(unused_variables)]
pub trait Inspect<'a>: Component {
	type SystemData: SystemData<'a>;
	const CAN_ADD: bool = false;
	const CAN_REMOVE: bool = true;

	fn inspect(data: &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {}
	fn can_add(data: &Self::SystemData, entity: Entity) -> bool { false }
	fn add(data: &Self::SystemData, entity: Entity) {}
	fn setup(data: &Self::SystemData, entity: Entity) {}
}

#[macro_export]
macro_rules! inspect_marker {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &Self::SystemData, entity: $crate::amethyst::ecs::Entity) { lazy.insert(entity, Self); }
		}
	};
}

#[macro_export]
macro_rules! inspect_default {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &Self::SystemData, entity: $crate::amethyst::ecs::Entity) { lazy.insert(entity, Self::default()); }
		}
	};
}

// TODO: renderer::Flipped
inspect_marker!(amethyst::renderer::Hidden);
inspect_marker!(amethyst::renderer::HiddenPropagate);
inspect_marker!(amethyst::renderer::ScreenSpace);
inspect_marker!(amethyst::renderer::Transparent);

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

			fn setup(&mut self, res: &mut Resources) {
				Self::SystemData::setup(res);
				let mut state = res.fetch_mut::<$crate::InspectorState>();
				state.prefabs = std::fs::read_dir("assets/prefabs").unwrap().map(|x| x.unwrap().file_name().into_string().unwrap()).collect();
			}

			$crate::paste::item! {
				fn run(&mut self, (mut inspector_state, lazy, entities, ($([<store $cmp>],)+), ($(mut [<data $cmp>],)+)): Self::SystemData) {
					amethyst_imgui::with(move |ui| {
						use $crate::amethyst_imgui::imgui::{self, im_str};
						use $crate::Inspect;

						ui.window(im_str!("Inspector"))
							.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
							.build(move || {
								if let Some(entity) = inspector_state.selected {
									if entities.is_alive(entity) {
										$($cmp::setup(&[<data $cmp>], entity);)+

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
												if ($cmp::CAN_ADD || $cmp::can_add(&[<data $cmp>], entity)) && ![<store $cmp>].contains(entity) {
													if ui.small_button(im_str!("{}", stringify!($cmp))) {
														$cmp::add(&[<data $cmp>], entity);
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
												if $cmp::CAN_REMOVE {
													ui.same_line(0.);
													remove = ui.small_button(im_str!("remove##{}_header_remove", stringify!($cmp)));
												}
												if remove {
													lazy.remove::<$cmp>(entity);
												} else if expanded {
													$cmp::inspect(&[<data $cmp>], entity, ui);
												}
											}
										)+

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
							});
					});
				}
			}
		}
	};
}
