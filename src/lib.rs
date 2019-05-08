#![allow(clippy::type_complexity, clippy::float_cmp)]

#[macro_use]
macro_rules! f {
	() => { || panic!("{}:{}", file!(), line!()) };
	(_) => { |_| panic!("{}:{}", file!(), line!()) };
}

#[macro_use]
macro_rules! inspect_enum {
	($current: expr, $label: expr, [$($variant:expr),+$(,)*]) => {{
		let mut current = 0;
		let source = vec![$($variant,)+];
		let size = source.len();
		let mut items = Vec::<imgui::ImString>::with_capacity(size);
		for (i, item) in source.iter().enumerate() {
			if *item == $current {
				current = i as i32;
			}
			items.push(im_str!("{:?}", item).into());
		}

		amethyst_imgui::with(|ui| {
			ui.combo($label, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), size as i32);
		});

		source[current as usize].clone()
	}};
}

pub use paste;
pub use amethyst_inspector_derive::*;
use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui::{self, im_str};

#[macro_use]
macro_rules! compare_fields {
	($first:expr, $second:expr, $($field:ident),+$(,)*) => (
		$($first.$field != $second.$field ||)+ false
	);
}

mod prelude;
mod hierarchy;
mod inspectors;
mod utils;
mod controls;

pub use hierarchy::InspectorHierarchy;
pub use inspectors::{SpriteRender::SpriteList, TextureHandle::TextureList, UiText::FontList, UiTransformDebug::UiTransformDebug};

#[allow(unused_variables)]
pub trait InspectControlBuilder<'control, 'resource: 'control, Value: InspectControl<'control, 'resource>>: Sized {
	fn new(value: Value) -> Self;
	fn data(self, data: &'control mut <Value as InspectControl<'control, 'resource>>::SystemData) -> Self { self }
	fn label(self, label: &'control imgui::ImStr) -> Self { self }
	fn build(self);
	fn changed(self, changed: &'control mut bool) -> Self { self }
}

/// Implement this on your fields to be able to `#[derive(Inspect)]` on your struct
pub trait InspectControl<'control, 'resource: 'control>: Sized {
	type SystemData: SystemData<'resource>;
	type Builder: InspectControlBuilder<'control, 'resource, Self>;

	fn control(self) -> Self::Builder {
		Self::Builder::new(self)
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
/// Whether the component is addable is decided by `can_add(...)`
#[allow(unused_variables)]
pub trait Inspect<'a>: Component {
	type SystemData: SystemData<'a>;

	/// This method is only ran if the component contains the selected entity
	fn inspect(data: &mut Self::SystemData, entity: Entity) {}
	/// Decide if this component can be added (e.g. because it requires another component)
	fn can_add(data: &mut Self::SystemData, entity: Entity) -> bool { false }
	/// Decide if this component can be removed (e.g. because it's required by another component)
	fn can_remove(data: &mut Self::SystemData, entity: Entity) -> bool { true }
	fn add(data: &mut Self::SystemData, entity: Entity) {}
	/// This method is ran on all entities, even if none are selected
	fn setup(data: &mut Self::SystemData, entity: Option<Entity>) {}
}

#[macro_export]
macro_rules! inspect_default {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = ::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>;

			fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
			fn add(lazy: &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { lazy.insert(entity, Self::default()); }
		}
	};
}

macro_rules! inspect_marker {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = ::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>;

			fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
			fn add(lazy: &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { lazy.insert(entity, Self); }
		}
	};
}

inspect_marker!(amethyst::renderer::Hidden);
inspect_marker!(amethyst::renderer::HiddenPropagate);
inspect_marker!(amethyst::renderer::ScreenSpace);
inspect_marker!(amethyst::renderer::Transparent);
#[cfg(saveload)]
inspect_marker!(amethyst::core::ecs::saveload::U64Marker);

impl<'a> Inspect<'a> for amethyst::renderer::Flipped {
	type SystemData = (Read<'a, LazyUpdate>, ReadStorage<'a, Self>);

	fn inspect((lazy, storage): &mut Self::SystemData, entity: Entity) {
		use amethyst::renderer::Flipped;

		let me = if let Some(x) = storage.get(entity) { *x } else { return; };
		let new_me = inspect_enum!(me, im_str!("flip"), [
			Flipped::None,
			Flipped::Horizontal,
			Flipped::Vertical,
			Flipped::Both,
		]);

		if me != new_me {
			lazy.insert(entity, new_me);
		}
	}

	fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
	fn add((lazy, ..): &mut Self::SystemData, entity: Entity) { lazy.insert(entity, amethyst::renderer::Flipped::None) }
}

#[macro_export]
macro_rules! inspector {
	($($cmp:ident),+$(,)*) => {
		use ::amethyst::{
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
					::amethyst_imgui::with(move |ui| {
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
												if <$cmp as Inspect>::can_add(&mut [<data $cmp>], entity) && ![<store $cmp>].contains(entity) {
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
												if <$cmp as Inspect>::can_remove(&mut [<data $cmp>], entity) {
													ui.same_line(0.);
													remove = ui.small_button(im_str!("remove##{}_header_remove", stringify!($cmp)));
												}
												if remove {
													lazy.remove::<$cmp>(entity);
												} else if expanded {
													<$cmp as Inspect>::inspect(&mut [<data $cmp>], entity);
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
