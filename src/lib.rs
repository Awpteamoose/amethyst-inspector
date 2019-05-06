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
// mod inspectors;
mod utils;
mod controls;

pub use hierarchy::InspectorHierarchy;
// pub use inspectors::{SpriteRender::SpriteList, TextureHandle::TextureList, UiText::FontList, UiTransformDebug::UiTransformDebug};

pub trait InspectControlBuilder<'small, 'big: 'small, Value: InspectControl<'small, 'big>>: Sized {
	fn new(value: Value) -> Self;
	fn data(self, data: &'small mut <Value as InspectControl<'small, 'big>>::SystemData) -> Self { self }
	fn label(self, label: &'small imgui::ImStr) -> Self { self }
	fn build(self);
	fn changed(self, changed: &'small mut bool) -> Self { self }
}

/// Implement this on your fields to be able to `#[derive(Inspect)]` on your struct
pub trait InspectControl<'small, 'big: 'small>: Sized + Send + Sync + 'small {
	type SystemData: SystemData<'big>;
	type Builder: InspectControlBuilder<'small, 'big, Self>;

	fn control(self) -> Self::Builder {
		Self::Builder::new(self)
	}
}

/*
/// Draggable uint as milliseconds
impl<'a> InspectControl<'a> for std::time::Duration {
	type SystemData = ();

	fn control(&mut self, data: &mut Self::SystemData, null_to: f32, speed: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
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
*/

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

#[derive(Clone, Default)]
pub struct Player {
	// #[inspect(skip)]
	pub location: LocationEntity,
	pub using_location: f32,
	// pub normul: Normul,
}

impl Component for Player {
	type Storage = DenseVecStorage<Self>;
}

#[derive(Default, Clone, PartialEq)]
pub struct LocationEntity(pub Option<Entity>);

impl<'small, 'big: 'small> InspectControl<'small, 'big> for &'small mut LocationEntity {
	type SystemData = (
		Entities<'big>,
	);
	type Builder = ThingBuilder<'small, 'big, Self>;
}

pub struct ThingBuilder<'small, 'big, Value: InspectControl<'small, 'big>> {
	pub value: &'small mut LocationEntity,
	pub label: Option<&'small imgui::ImStr>,
	pub changed: Option<&'small mut bool>,
	pub data: Option<&'small mut <Value as InspectControl<'small, 'big>>::SystemData>,
}

impl<'small, 'big: 'small> InspectControlBuilder<'small, 'big, &'small mut LocationEntity> for ThingBuilder<'small, 'big, &'small mut LocationEntity> {
	fn new(value: &'small mut LocationEntity) -> Self {
		Self { value, label: None, changed: None, data: None }
	}
	// fn label(mut self, label: &'a imgui::ImStr) -> Self {
	//	   self.label = Some(label);
	//	   self
	// }
	// fn changed(mut self, changed: &'a mut bool) -> Self {
	//	   self.changed = Some(changed);
	//	   self
	// }
	// fn data(mut self, data: &'a mut <LocationEntity as InspectControl<'a>>::SystemData) -> Self {
	//	   self.data = Some(data);
	//	   self
	// }
	fn build(self) {
		amethyst_imgui::with(|ui| {
			// let data = self.data.unwrap();
			// let mut changed = false;
			// let mut current = 0;
			// let list = std::iter::once(None).chain((&data.0, &data.2).join().map(|x| Some(x.1))).collect::<Vec<_>>();
			// let mut items = Vec::<imgui::ImString>::new();
			// for (i, &entity) in list.iter().enumerate() {
			//	   if *self.value == LocationEntity(entity) { current = i as i32; }

			//	   let label: String = if let Some(entity) = entity {
			//		   if let Some(name) = data.1.get(entity) {
			//			   name.name.to_string()
			//		   } else {
			//			   format!("Entity {}/{}", entity.id(), entity.gen().id())
			//		   }
			//	   } else {
			//		   "None".into()
			//	   };
			//	   items.push(imgui::im_str!("{}", label).into());
			// }
			// changed = ui.combo(imgui::im_str!("location"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
			// *self.value = LocationEntity(list[current as usize]);

			// let mut v = *self.value as _;
			// let mut changed = ui.[<drag_$kind>](self.label.unwrap(), &mut v).speed(self.speed).min(std::$type::MIN as _).max(std::$type::MAX as _).build();
			// *self.value = v as _;
			// if ui.is_item_hovered() && ui.imgui().is_mouse_down(imgui::ImMouseButton::Right) {
			//	   changed = true;
			//	   *self.value = self.null_to;
			// }
			// if let Some(x) = self.changed { *x = *x || changed };
		});
	}
}

impl<'a> Inspect<'a> for Player {
	type SystemData = (
		::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>,
		::amethyst::ecs::ReadStorage<'a, Self>,
		<&'a mut LocationEntity as InspectControl<'a, 'a>>::SystemData,
		// <f32 as InspectControl<'a>>::SystemData,
	);
	fn inspect(
		(lazy, storage, systemdata_location): &mut Self::SystemData,
		entity: ::amethyst::ecs::Entity,
	) {
		::amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) {
				x
			} else {
				return;
			};
			let mut new_me = me.clone();
			let mut changed = false;
			ui.push_id(::amethyst_imgui::imgui::im_str!("{}", stringify!(Player)));
			<&mut LocationEntity as InspectControl>::control(&mut new_me.location)
				.changed(&mut changed)
				.data(systemdata_location)
				.label(::amethyst_imgui::imgui::im_str!("{}", stringify!(location)))
				.build();
			// <f32 as InspectControl>::control(&mut new_me.using_location)
			//     .changed(&mut changed)
			//     .data(systemdata_using_location)
			//     .label(::amethyst_imgui::imgui::im_str!(
			//         "{}",
			//         stringify!(using_location)
			//     ))
			//     .build();
			if changed {
				lazy.insert(entity, new_me);
			}
			ui.pop_id();
		});
	}
	// fn add((lazy, ..): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) {
	//     lazy.insert(entity, Self::default());
	// }
	// fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool {
	//     true
	// }
}

// #[derive(Serialize, Deserialize, Clone, Debug, Default)]
// pub struct Normul;

// impl<'a> amethyst_inspector::InspectControl<'a> for Normul {
//	   type SystemData = (
//		   r!(cmp::Player),
//		   r!(cmp::Location),
//		   r!(cmp::Named),
//		   r!(res::Entities),
//	   );

//	   fn control(&mut self, _: f32, _: f32, label: &imgui::ImStr, ui: &imgui::Ui<'_>) -> bool {
//		   let me = if let Some(x) = storage.get(entity) { x } else { return; };
//		   let mut new_me = me.clone();
//		   let mut changed = false;
//		   ui.push_id(im_str!("Player"));

//		   {
//			   let mut current = 0;
//			   let list = std::iter::once(None).chain((&*location_s, &*entities).join().map(|x| Some(x.1))).collect::<Vec<_>>();
//			   let mut items = Vec::<imgui::ImString>::new();
//			   for (i, &entity) in list.iter().enumerate() {
//				   if me.location == entity { current = i as i32; }

//				   let label: String = if let Some(entity) = entity {
//					   if let Some(name) = named_s.get(entity) {
//						   name.name.to_string()
//					   } else {
//						   format!("Entity {}/{}", entity.id(), entity.gen().id())
//					   }
//				   } else {
//					   "None".into()
//				   };
//				   items.push(imgui::im_str!("{}", label).into());
//			   }
//			   changed = ui.combo(imgui::im_str!("location"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
//			   new_me.location = list[current as usize];
//		   }
//	   }
// }

// impl<'a> Inspect<'a> for Player {
//	   type SystemData = (
//		   r!(LazyUpdate),
//		   r!(cmp::Player),
//		   r!(cmp::Location),
//		   r!(cmp::Named),
//		   r!(res::Entities),
//	   );

//	   fn can_add(_: &mut Self::SystemData, _: Entity) -> bool { true }
//	   fn inspect((lazy, storage, location_s, named_s, entities): &mut Self::SystemData, entity: Entity) {
//		   amethyst_imgui::with(|ui| {
//			   let me = if let Some(x) = storage.get(entity) { x } else { return; };
//			   let mut new_me = me.clone();
//			   let mut changed = false;
//			   ui.push_id(im_str!("Player"));

//			   {
//				   let mut current = 0;
//				   let list = std::iter::once(None).chain((&*location_s, &*entities).join().map(|x| Some(x.1))).collect::<Vec<_>>();
//				   let mut items = Vec::<imgui::ImString>::new();
//				   for (i, &entity) in list.iter().enumerate() {
//					   if me.location == entity { current = i as i32; }

//					   let label: String = if let Some(entity) = entity {
//						   if let Some(name) = named_s.get(entity) {
//							   name.name.to_string()
//						   } else {
//							   format!("Entity {}/{}", entity.id(), entity.gen().id())
//						   }
//					   } else {
//						   "None".into()
//					   };
//					   items.push(imgui::im_str!("{}", label).into());
//				   }
//				   changed = ui.combo(imgui::im_str!("location"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
//				   new_me.location = list[current as usize];
//			   }

//			   new_me.using_location.control().null_to(0.).speed(0.).label(im_str!("using_location")).changed(&mut changed).build();

//			   if changed {
//				   lazy.insert(entity, new_me);
//			   }

//			   ui.pop_id();
//		   });
//	   }
// }
