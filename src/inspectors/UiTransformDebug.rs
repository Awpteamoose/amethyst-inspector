use amethyst::{
	core::{transform::Transform, Named},
	ecs::prelude::*,
	renderer::Rgba,
	ui::UiTransform,
};
use amethyst_imgui::imgui;
use crate::Inspect;
use imgui::im_str;

#[derive(Clone, PartialEq)]
pub struct UiTransformDebug {
	camera: Entity,
	color: Rgba,
	children: bool,
	always: bool,
}
impl Component for UiTransformDebug {
	type Storage = amethyst::ecs::DenseVecStorage<Self>;
}

impl<'a> Inspect<'a> for UiTransformDebug {
	type SystemData = (
		ReadStorage<'a, Self>,
		ReadStorage<'a, UiTransform>,
		ReadStorage<'a, Transform<f32>>,
		ReadStorage<'a, amethyst::core::Parent>,
		ReadExpect<'a, amethyst::core::ParentHierarchy>,
		ReadExpect<'a, amethyst::renderer::ScreenDimensions>,
		ReadStorage<'a, Named>,
		ReadStorage<'a, amethyst::renderer::Camera>,
		Entities<'a>,
		Read<'a, LazyUpdate>,
	);

	fn setup((storage, ui_transforms, transforms, _, _, dimensions, _, cameras, entities, lazy): &mut Self::SystemData, inspectee: Option<Entity>) {
		for (debug, entity) in (&*storage, &*entities).join() {
			if Some(entity) != inspectee && !debug.always { return; };
			let transform = if let Some(x) = ui_transforms.get(entity) { x } else { return; };
			let camera = if let Some(x) = cameras.get(debug.camera) { x } else { return; };
			let camera_transform = if let Some(x) = transforms.get(debug.camera) { x } else { return; };

			let matrix = camera_transform.matrix();
			let x = transform.pixel_x();
			let y = dimensions.height() - transform.pixel_y();
			let z = transform.local_z;
			let w = transform.width;
			let h = transform.height;

			let mut points = [
				camera.position_from_screen([x - w * 0.5, y - h * 0.5].into(), &matrix, dimensions),
				camera.position_from_screen([x + w * 0.5, y - h * 0.5].into(), &matrix, dimensions),
				camera.position_from_screen([x + w * 0.5, y + h * 0.5].into(), &matrix, dimensions),
				camera.position_from_screen([x - w * 0.5, y + h * 0.5].into(), &matrix, dimensions),
			];
			for p in points.iter_mut() {
				p[2] = z;
			}

			let color = debug.color;
			lazy.exec(move |w| {
				let mut lines = w.write_resource::<amethyst::renderer::DebugLines>();
				lines.draw_line(points[0], points[1], color);
				lines.draw_line(points[1], points[2], color);
				lines.draw_line(points[2], points[3], color);
				lines.draw_line(points[3], points[0], color);
			});
		}
	}

	fn inspect((storage, _, _, _, _, _, names, cameras, entities, lazy): &mut Self::SystemData, entity: Entity) {
		amethyst_imgui::with(|ui| {
			let me = if let Some(x) = storage.get(entity) { x } else { return; };
			let mut new_me = me.clone();
			ui.push_id(im_str!("ui_transform_debug"));

			let camera_entities = (&*cameras, &*entities).join().map(|(_, e)| e).collect::<Vec<Entity>>();
			if camera_entities.len() > 1 {
				let mut current = 0;
				let mut items = Vec::<imgui::ImString>::with_capacity(camera_entities.len());
				for (i, &camera_entity) in camera_entities.iter().enumerate() {
					if me.camera == camera_entity {
						current = i as i32;
					}

					let label: String = if let Some(name) = names.get(camera_entity) {
						name.name.to_string()
					} else {
						format!("Entity {}/{}", camera_entity.id(), camera_entity.gen().id())
					};
					items.push(im_str!("{}", label).into());
				}

				ui.combo(im_str!("camera"), &mut current, items.iter().map(std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10);
				new_me.camera = camera_entities[current as usize];
			}

			let mut v: [f32; 4] = new_me.color.into();
			ui.drag_float4(im_str!("colour"), &mut v)
				.speed(0.005)
				.build();
			new_me.color = v.into();
			ui.checkbox(im_str!("children"), &mut new_me.children);
			ui.checkbox(im_str!("always"), &mut new_me.always);

			if *me != new_me {
				lazy.insert(entity, new_me);
			}
			ui.pop_id();
		});
	}

	fn can_add((_, ui_transforms, _, _, _, _, _, cameras, entities, _): &mut Self::SystemData, entity: Entity) -> bool {
		(&*cameras, &*entities).join().nth(0).is_some() && ui_transforms.contains(entity)
	}

	fn add((_, _, _, _, _, _, _, cameras, entities, lazy): &mut Self::SystemData, entity: Entity) {
		let (_, camera) = if let Some(x) = (&*cameras, &*entities).join().nth(0) { x } else { return; };
		lazy.insert(entity, UiTransformDebug { camera, color: Rgba::red(), children: false, always: true });
	}
}
