## About:
Unity-inspired entity hierarchy and component editor via [amethyst-imgui](https://github.com/Awpteamoose/amethyst-imgui)

## Usage:
1. Implement `Inspect` for all components that you want to show up in the inspector. For example:
```rust
impl<'a> Inspect<'a> for Transform {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let mut me = if let Some(x) = storage.get(entity) { x.clone() } else { return; };

		{
			let translation = me.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			me.set_translation(v.into());
		}

		{
			let mut rotation = me.rotation().euler_angles().2.to_degrees();
			if rotation == -180. {
				rotation = 180.;
			}
			ui.drag_float(imgui::im_str!("rotation##transform{:?}", entity), &mut rotation)
				.speed(0.25)
				.build();
			me.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = me.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{:?}", entity), &mut v)
				.speed(0.1)
				.build();
			me.set_scale(v[0], v[1], 1.);
		}

		lazy.insert(entity, me);
	}
}
```
2. Run `inspect_marker!` macro over your marker components.
```rust
pub struct MarkerCmp;
impl Component for MarkerCmp {
	type Storage = DenseVecStorage<Self>;
}

inspect_marker!(MarkerCmp);
```
3. List all your components you want to show up in the inspector with an `inspector!` macro. This creates a system called `Inspector`.
```rust
inspector![
	Named,
	Transform,
	Rgba,
];
```
3. Add `InspectorHierarchy` and `Inspector` systems as late as possible, ideally right before `amethyst_imgui::EndFrame`.
```rust
	.with(InspectorHierarchy::<UserData>::default(), "inspector_hierarchy", &[])
	.with(Inspector, "inspector", &["inspector_hierarchy"])
	.with_barrier()
	.with(amethyst_imgui::EndFrame::default(), "imgui_end", &[]);
```

# Add/remove components
You can enable your components to be added (off by default) or removed (on by default) by specifying `CAN_ADD` and an `add` method or `CAN_REMOVE` for removal.
```rust
impl<'a> Inspect<'a> for Named {
	type SystemData = (ReadStorage<'a, Self>, Read<'a, LazyUpdate>);

	const CAN_ADD: bool = true;

	fn inspect((storage, lazy): &Self::SystemData, entity: Entity, ui: &imgui::Ui<'_>) {
		let me = if let Some(x) = storage.get(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.input_text(imgui::im_str!("Entity {}/{}##named", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();

		lazy.insert(entity, Named::new(buf.to_str().to_owned()));
	}

	fn add((_storage, lazy): &Self::SystemData, entity: Entity) {
		lazy.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id())));
	}
}
```

# Help wanted
Drop me a line on discord or create an issue if you can help or have advice:

* Derive macro for simple inspectors, also to replace `inspect_marker!`
* Make a modal or smth specifying properties when adding components
* Create/remove entities
* Reparent entities (at least via inspector menu)
* Save/load entities
	* somehow must be able to specify resource handles and whatnot

![screenshot](https://raw.githubusercontent.com/awpteamoose/amethyst-inspector/master/screenshot.png)
