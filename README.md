## About:
Unity-inspired entity hierarchy and component editor via [amethyst-imgui](https://github.com/Awpteamoose/amethyst-imgui)

## Usage:
1. Implement `Inspect` for all components that you want to show up in the inspector. For example:
```rust
impl<'a> Inspect<'a> for Transform {
	type UserData = &'a mut dyn Any;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };

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
			ui.drag_float(
				imgui::im_str!("rotation##transform{:?}", entity),
				&mut rotation,
			)
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
3. List your `UserData` and all components you want to show up in the inspector with an `inspector!` macro. This creates a system called `Inspector`. For example:
```rust
#[derive(Default)]
pub struct UserData;

inspector![
	UserData,
	Named,
	Transform,
	Rgba,
];
```
3. Add `InspectorHierarchy` and `Inspector` to your systems
```rust
	.with(InspectorHierarchy::<UserData>::default(), "inspector_hierarchy", &[])
	.with(Inspector, "inspector", &["inspector_hierarchy"])
```

# Add/remove components
You can enable your components to be added (off by default) or removed (on by default) by specifying `CAN_ADD` and an `add` method or `CAN_REMOVE` for removal.
```rust
impl<'a> Inspect<'a> for Named {
	type UserData = &'a mut dyn Any;
	const CAN_ADD: bool = true;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };
		let mut buf = imgui::ImString::new(me.name.clone());
		ui.input_text(imgui::im_str!("Entity {}/{}##named", entity.id(), entity.gen().id()), &mut buf)
			.resize_buffer(true)
			.build();
		me.name = std::borrow::Cow::from(String::from(buf.to_str()));
	}

	fn add(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		storage.insert(entity, Named::new(format!("Entity {}/{}", entity.id(), entity.gen().id()))).unwrap();
	}
}
```

# Sharing component data
Your `UserData` struct can be used by inspectors to pass some data along. For example:
```rust
// Add this to entities using SpriteRender
pub struct SpriteInfo(pub u32, pub u32);
impl Component for SpriteInfo {
	type Storage = DenseVecStorage<Self>;
}

// ------------------

struct UserData {
	first_sprite: i32,
	last_sprite: i32,
}

// ------------------

impl<'a> Inspect<'a> for SpriteInfo {
	type UserData = &'a mut UserData;

	fn setup(storage: &mut WriteStorage<'_, Self>, entity: Entity, user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };
		user_data.first_sprite = me.0 as i32;
		user_data.last_sprite = me.1 as i32;
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::SpriteRender {
	type UserData = &'a mut UserData;

	fn inspect(storage: &mut WriteStorage<'_, Self>, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		let me = if let Some(x) = storage.get_mut(entity) { x } else { return; };
		let mut sprite_number = me.sprite_number as i32;
		ui.slider_int(imgui::im_str!("# sprite##sprite_render{}{}", entity.id(), entity.gen().id()), &mut sprite_number, user_data.first_sprite, user_data.last_sprite).build();
		me.sprite_number = sprite_number as usize;
		ui.separator();
	}
}

// ------------------

inspector![
	UserData,
	Named,
	Transform,
	Rgba,
	SpriteInfo,
	SpriteRender,
];
```

# Help wanted
Drop me a line on discord or create an issue if you can help or have advice:

* Derive macro for simple inspectors, also to replace `inspect_marker!`
	* if possible, should also automatically specify `CAN_ADD` and `add` if the component implements `Default`
* Make a modal or smth specifying properties when adding components
* Create/remove entities
* Reparent entities (at least via inspector menu)
* Somehow get access to storages to be able to more easily use data form other componens and get rid of `UserData`
* Save/load entities
	* idk it doesn't look like amethyst's prefabs can save
	* since everything is running in a system, somehow have to be able to specify resource handles and whatnot

![screenshot](https://raw.githubusercontent.com/awpteamoose/amethyst-inspector/master/screenshot.png)
