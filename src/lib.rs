pub use amethyst;
use amethyst::{
	core::{transform::Transform, Named},
	ecs::prelude::*,
	renderer::{Blink, Hidden, HiddenPropagate, Rgba, SpriteRender},
};
pub use amethyst_imgui;
use amethyst_imgui::imgui;
pub use paste;

#[derive(Default)]
pub struct InspectorState {
	// pub to_save: Vec<Entity>,
	pub selected: Option<Entity>,
}

#[derive(Default, Clone, Copy)]
pub struct InspectorHierarchy;
impl<'s> System<'s> for InspectorHierarchy {
	type SystemData = (
		Write<'s, InspectorState>,
		ReadStorage<'s, amethyst::core::Named>,
		ReadStorage<'s, amethyst::core::Parent>,
		ReadExpect<'s, amethyst::core::ParentHierarchy>,
		Entities<'s>,
		Read<'s, LazyUpdate>,
	);

	fn run(&mut self, (mut inspector_state, names, parents, hierarchy, entities, lazy): Self::SystemData) {
		amethyst_imgui::with(move |ui| {
			ui.window(imgui::im_str!("Hierarchy"))
				.size((300.0, 500.0), imgui::ImGuiCond::FirstUseEver)
				.build(move || {
					fn render_boy(
						entity: Entity,
						hierarchy: &amethyst::core::ParentHierarchy,
						names: &ReadStorage<'_, amethyst::core::Named>,
						ui: &imgui::Ui<'_>,
						inspector_state: &mut InspectorState,
						entities: &amethyst::ecs::world::EntitiesRes,
						lazy: &LazyUpdate,
					) {
						let children = hierarchy.children(entity);

						let label: String = if let Some(name) = names.get(entity) {
							name.name.to_string()
						} else {
							format!("Entity {}/{}", entity.id(), entity.gen().id())
						};

						macro_rules! tree_node_buttons {
							() => {
								ui.same_line(0.);
								if ui.small_button(imgui::im_str!("inspect##selector{:?}", entity)) {
									inspector_state.selected = Some(entity);
								}
							};
						}

						let mut opened = false;
						ui.tree_node(imgui::im_str!("{}##{:?}", label, entity))
							.label(imgui::im_str!("{}", label))
							.selected(matches::matches!(inspector_state.selected, Some(x) if x == entity))
							.leaf(children.is_empty())
							.build(|| {
								opened = true;
								tree_node_buttons!();
								for child in children {
									render_boy(*child, hierarchy, names, ui, inspector_state, &entities, &lazy);
								}
							});

						if !opened {
							tree_node_buttons!();
						}
					};

					if ui.small_button(imgui::im_str!("new entity##hierarchy")) {
						lazy.create_entity(&entities).build();
					}
					ui.separator();
					for (entity, _) in (&entities, !&parents).join() {
						render_boy(entity, &hierarchy, &names, &ui, &mut inspector_state, &entities, &lazy);
					}
				});
		});
	}
}

pub trait Inspect<'a>: Component {
	type SystemData: SystemData<'a>;
	const CAN_ADD: bool = false;
	const CAN_REMOVE: bool = true;

	fn inspect(data: &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {}
	fn add(data: &Self::SystemData, entity: Entity) {}
	fn setup(data: &Self::SystemData, entity: Entity) {}
}

impl<'a> Inspect<'a> for Named {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.input_text(imgui::im_str!("Entity {}/{}##named", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();

		let new_name = buf.to_str().to_owned();
		if me.name != new_name {
			lazy.insert(entity, Named::new(new_name));
		}
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id())));
	}
}

impl<'a> Inspect<'a> for Transform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut new_me = me.clone();

		{
			let translation = new_me.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			new_me.set_translation(v.into());
		}

		{
			let mut rotation = new_me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			ui.drag_float(imgui::im_str!("rotation##transform{:?}", entity), &mut rotation)
				.speed(0.25)
				.build();
			new_me.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = new_me.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			new_me.set_scale(v[0], v[1], 1.);
		}

		if *me != new_me {
			lazy.insert(entity, new_me);
		}

		// let mut current_entity
		// let parent = parents.get(entitiy);

		// for (i, entity) in entities.join().enumerate() {
		//     if let Some(e) = parent {
		//         if e == entity {
		//             current_entity = i;
		//         }
		//     }
		//     if entity == Some(parent)
		// }
		// ui.combo(imgui::im_str!("parent##transform{:?}", entity), &mut current_item, &items, height_in_items: 10);
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Transform::default());
	}
}

impl<'a> Inspect<'a> for SpriteRender {
	type SystemData = (
		ReadStorage<'a, Self>,
		ReadExpect<'a, amethyst::assets::AssetStorage<amethyst::renderer::SpriteSheet>>,
		Read<'a, LazyUpdate>,
	);

	fn inspect((storage, sprites, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let mut me = if let Some(x) = storage.get(entity) {
			x.clone()
		} else {
			return;
		};

		let mut sprite_number = me.sprite_number as i32;
		ui.slider_int(
			imgui::im_str!("# sprite##sprite_render{:?}", entity),
			&mut sprite_number,
			0,
			sprites.get(&me.sprite_sheet).unwrap().sprites.len() as i32 - 1,
		)
		.build();

		if me.sprite_number != sprite_number as usize {
			me.sprite_number = sprite_number as usize;
			lazy.insert(entity, me);
		}
	}
}

impl<'a> Inspect<'a> for Rgba {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) {
			x
		} else {
			return;
		};

		let mut v: [f32; 4] = [me.0, me.1, me.2, me.3];
		ui.drag_float4(imgui::im_str!("colour tint##rgba{:?}", entity), &mut v)
			.speed(0.1)
			.build();

		if *me != Rgba::from(v) {
			lazy.insert(entity, Rgba::from(v));
		}
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, amethyst::renderer::Rgba::white());
	}
}

impl<'a> Inspect<'a> for Blink {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let &Blink {
			mut delay,
			timer,
			mut absolute_time,
		} = if let Some(x) = storage.get(entity) {
			x
		} else {
			return;
		};
		ui.drag_float(imgui::im_str!("delay##blink{:?}", entity), &mut delay)
			.speed(0.1)
			.build();
		ui.checkbox(imgui::im_str!("absolute time##blink{:?}", entity), &mut absolute_time);

		lazy.insert(
			entity,
			Blink {
				delay,
				timer,
				absolute_time,
			},
		);
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(
			entity,
			Blink {
				delay: 0.5,
				timer: 0.,
				absolute_time: false,
			},
		);
	}
}

#[macro_export]
macro_rules! inspect_marker {
	($cmp: ident) => {
		impl<'a> $crate::Inspect<'a> for $cmp {
			type SystemData = $crate::amethyst::ecs::Read<'a, $crate::amethyst::ecs::LazyUpdate>;

			const CAN_ADD: bool = true;

			fn add(lazy: &Self::SystemData, entity: $crate::amethyst::ecs::Entity) {
				lazy.insert(entity, $cmp);
			}
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
										if $cmp::CAN_ADD && ![<store $cmp>].contains(entity) {
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
