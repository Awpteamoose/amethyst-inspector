#![allow(clippy::type_complexity, clippy::float_cmp)]

pub use amethyst;
use amethyst::{
	ecs::prelude::*,
	renderer::{Hidden, HiddenPropagate},
};
pub use amethyst_imgui;
use amethyst_imgui::imgui;
pub use paste;

#[macro_use]
macro_rules! compare_fields {
	($first:expr, $second:expr, $($field:ident),+$(,)*) => (
		$($first.$field != $second.$field ||)+ false
	);
}

mod hierarchy;
mod inspectors;

pub use hierarchy::*;
pub use inspectors::{SpriteRender::SpriteList, TextureHandle::TextureList, UiText::FontList, UiTransformDebug::*};

#[derive(Default)]
pub struct InspectorState {
	// pub to_save: Vec<Entity>,
	pub selected: Option<Entity>,
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
	($cmp: ident) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &Self::SystemData, entity: $crate::amethyst::ecs::Entity) { lazy.insert(entity, $cmp); }
		}
	};
}

inspect_marker!(Hidden);
inspect_marker!(HiddenPropagate);

#[macro_export]
macro_rules! inspector {
	($($cmp:ident),+$(,)*) => {
		#[derive(Default)]
		#[allow(missing_copy_implementations)]
		pub struct Inspector;
		impl<'s> System<'s> for Inspector {
			type SystemData = (
				$crate::amethyst::ecs::Write<'s, $crate::InspectorState>,
				$crate::amethyst::ecs::Read<'s, $crate::amethyst::ecs::LazyUpdate>,
				$crate::amethyst::ecs::Entities<'s>,
				($($crate::amethyst::ecs::ReadStorage<'s, $cmp>,)+),
				($(<$cmp as $crate::Inspect<'s>>::SystemData,)+),
			);

			$crate::paste::item! {
				#[allow(non_snake_case)]
				fn run(&mut self, (mut inspector_state, lazy, entities, ($([<store $cmp>],)+), ($(mut [<data $cmp>],)+)): Self::SystemData) {
					amethyst_imgui::with(move |ui| {
						use $crate::amethyst_imgui::imgui;
						use $crate::Inspect;

						ui.window(imgui::im_str!("Inspector"))
							.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
							.build(move || {
								let entity = if let Some(x) = inspector_state.selected { x } else { return; };
								if !entities.is_alive(entity) { return; }
								$($cmp::setup(&mut [<data $cmp>], entity);)+

								if ui.small_button(imgui::im_str!("make child##inspector{:?}", entity)) {
									lazy.create_entity(&entities)
										.with(amethyst::core::transform::Parent::new(entity))
										.build();
								}
								ui.same_line(0.);
								if ui.small_button(imgui::im_str!("remove##inspector{:?}", entity)) {
									lazy.exec_mut(move |w| w.delete_entity(entity).unwrap());
								}

								if ui.collapsing_header(imgui::im_str!("add component##{:?}", entity)).build() {
									let mut hor_pos = 0.;
									$(
										if ($cmp::CAN_ADD || $cmp::can_add(&mut [<data $cmp>], entity)) && ![<store $cmp>].contains(entity) {
											if ui.small_button(imgui::im_str!("{}", stringify!($cmp))) {
												$cmp::add(&mut [<data $cmp>], entity);
											}
											hor_pos += ui.get_item_rect_size().0 + ui.imgui().style().item_spacing.x;
											if hor_pos < ui.get_content_region_avail().0 {
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
										let expanded = ui.collapsing_header(imgui::im_str!("{}##header{:?}", stringify!($cmp), entity)).flags(imgui::ImGuiTreeNodeFlags::AllowItemOverlap).default_open(true).build();
										if $cmp::CAN_REMOVE {
											ui.same_line(0.);
											remove = ui.small_button(imgui::im_str!("remove##{}_header_remove", stringify!($cmp)));
										}
										if remove {
											lazy.remove::<$cmp>(entity);
										} else if expanded {
											$cmp::inspect(&mut [<data $cmp>], entity, ui);
										}
									}
								)+

								// ui.separator();

								// if ui.small_button(imgui::im_str!("save##inspector")) {
								//     inspector_state.to_save.push(entity);
								// }
							});
					});
				}
			}
		}
	};
}
