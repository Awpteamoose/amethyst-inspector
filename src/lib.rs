#![allow(clippy::type_complexity, clippy::float_cmp)]

#[macro_use]
macro_rules! f {
	() => { || panic!("{}:{}", file!(), line!()) };
	(_) => { |_| panic!("{}:{}", file!(), line!()) };
}

#[macro_use]
macro_rules! inspect_enum {
	($ui: expr, $current: expr, $label: expr, $changed: expr, [$($variant:expr),+$(,)*]) => {{
		let mut current = 0;
		let source = vec![$($variant,)+];
		let size = source.len();
		let mut items = Vec::<imgui::ImString>::with_capacity(size);
		for (i, item) in source.iter().enumerate() {
			if *item == $current {
				current = i;
			}
			items.push(im_str!("{:?}", item).into());
		}

		// TODO: regular combo
		$changed = imgui::ComboBox::new($label).build_simple_string($ui, &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice()) || $changed;

		source[current as usize].clone()
	}};
}

pub use paste;
pub use amethyst_inspector_derive::*;
use amethyst::ecs::prelude::*;
use amethyst_imgui::imgui;

mod prelude;
mod hierarchy;
mod inspectors;
mod controls;

pub use hierarchy::InspectorHierarchy;
pub use inspectors::{SpriteRender::SpriteList, UiText::FontList};

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
	pub selected: Option<Entity>,
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

#[macro_export]
macro_rules! inspect_marker {
	($cmp: path) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = ::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>;

			fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
			fn add(lazy: &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { lazy.insert(entity, Self); }
		}
	};
}

inspect_marker!(amethyst::core::Hidden);
inspect_marker!(amethyst::core::HiddenPropagate);
// inspect_marker!(amethyst::renderer::ScreenSpace);
inspect_marker!(amethyst::renderer::Transparent);

// impl<'a> Inspect<'a> for amethyst::renderer::Flipped {
//     type SystemData = (Read<'a, LazyUpdate>, ReadStorage<'a, Self>);

//     fn inspect((lazy, storage): &mut Self::SystemData, entity: Entity) {
//         use amethyst::renderer::Flipped;

//         let me = if let Some(x) = storage.get(entity) { *x } else { return; };
//         let new_me = inspect_enum!(me, im_str!("flip"), [
//             Flipped::None,
//             Flipped::Horizontal,
//             Flipped::Vertical,
//             Flipped::Both,
//         ]);

//         if me != new_me {
//             lazy.insert(entity, new_me);
//         }
//     }

//     fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { true }
//     fn add((lazy, ..): &mut Self::SystemData, entity: Entity) { lazy.insert(entity, amethyst::renderer::Flipped::None) }
// }

#[doc(hidden)]
pub fn draw_add_component<'a, Component: Inspect<'a>>(
	ui: &imgui::Ui,
	name: &str,
	cmp_data: &mut Component::SystemData,
	store: &ReadStorage<'_, Component>,
	entity: Entity,
	hor_pos: &mut f32,
) {
	if Component::can_add(cmp_data, entity) && !store.contains(entity) {
		if ui.small_button(&imgui::im_str!("{}", name)) {
			Component::add(cmp_data, entity);
		}
		*hor_pos += ui.item_rect_size()[0] + ui.clone_style().item_spacing[0];
		if *hor_pos + ui.item_rect_size()[0] < ui.content_region_avail()[0] {
			ui.same_line(0.);
		} else {
			*hor_pos = 0.;
		}
	}
}

#[doc(hidden)]
pub fn draw_inspect_component<'a, Component: Inspect<'a> + Send + Sync>(
	ui: &imgui::Ui,
	name: &str,
	cmp_data: &mut Component::SystemData,
	store: &ReadStorage<'_, Component>,
	entity: Entity,
	lazy: &Read<'_, LazyUpdate>,
) {
	if store.contains(entity) {
		let mut remove = false;
		let expanded = ui.collapsing_header(&imgui::im_str!("{}##header{:?}", name, entity)).flags(imgui::ImGuiTreeNodeFlags::AllowItemOverlap).default_open(true).build();
		if Component::can_remove(cmp_data, entity) {
			ui.same_line(0.);
			remove = ui.small_button(&imgui::im_str!("remove##{}_header_remove", name));
		}
		if remove {
			lazy.remove::<Component>(entity);
		} else if expanded {
			Component::inspect(cmp_data, entity);
		}
	}
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

			$crate::paste::item! {
				fn run(&mut self, (mut inspector_state, lazy, entities, ($([<store $cmp>],)+), ($(mut [<data $cmp>],)+)): Self::SystemData) {
					::amethyst_imgui::with(move |ui| {
						use ::amethyst_imgui::imgui::{self, im_str};
						use $crate::Inspect;

						imgui::Window::new(&im_str!("Inspector"))
							.size([300.0, 500.0], imgui::Condition::FirstUseEver)
							.build(ui, move || {
								$(<$cmp as Inspect>::setup(&mut [<data $cmp>], inspector_state.selected);)+
								if let Some(entity) = inspector_state.selected {
									if entities.is_alive(entity) {
										if ui.small_button(&im_str!("make child##inspector{:?}", entity)) {
											lazy.create_entity(&entities)
												.with(amethyst::core::transform::Parent::new(entity))
												.build();
										}
										ui.same_line(0.);
										if ui.small_button(&im_str!("remove##inspector{:?}", entity)) {
											lazy.exec_mut(move |w| w.delete_entity(entity).unwrap());
										}

										if ui.collapsing_header(&im_str!("add component")).build() {
											let mut hor_pos = 0.;
											$($crate::draw_add_component(ui, stringify!($cmp), &mut [<data $cmp>], &[<store $cmp>], entity, &mut hor_pos);)+
											if hor_pos > 0. {
												ui.new_line();
											}

											ui.separator();
										}

										$($crate::draw_inspect_component(ui, stringify!($cmp), &mut [<data $cmp>], &[<store $cmp>], entity, &lazy);)+
									}
								}
							});
					});
				}
			}
		}
	};
}
