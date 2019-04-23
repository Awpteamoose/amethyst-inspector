## About:
Unity-inspired entity hierarchy and component editor via [amethyst-imgui](https://github.com/Awpteamoose/amethyst-imgui)

## Usage:
1. Implement `Inspect` for all components that you want to show up in the inspector. For example:
```rust
impl<'a> Inspect<'a> for Transform {
	type UserData = &'a mut dyn Any;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, _user_data: Self::UserData) {
		{
			let translation = self.translation();
			let mut v: [f32; 3] = [translation[0], translation[1], translation[2]];
			ui.drag_float3(imgui::im_str!("translation##transform{}{}", entity.id(), entity.gen().id()), &mut v).speed(0.1).build();
			self.set_translation(v.into());
		}

		{
			let mut rotation = self.rotation().euler_angles().2.to_degrees();
			if rotation == -180. { rotation = 180.; }
			ui.drag_float(imgui::im_str!("rotation##transform{}{}", entity.id(), entity.gen().id()), &mut rotation).speed(0.25).build();
			self.set_rotation_2d(rotation.to_radians());
		}

		{
			let scale = self.scale().xy();
			let mut v: [f32; 2] = [scale[0], scale[1]];
			ui.drag_float2(imgui::im_str!("scale##transform{}{}", entity.id(), entity.gen().id()), &mut v).speed(0.1).build();
			self.set_scale(v[0], v[1], 1.);
		}
		ui.separator();
	}
}
```
2. List your `UserData` and all components you want to show up in the inspector with an `inspector!` macro. This creates a system called `Inspector`. For example:
```rust
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
	.with(InspectorHierarchy, "inspector_hierarchy", &[])
	.with(Inspector, "inspector", &["inspector_hierarchy"])
```

# Advanced usage:
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
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		user_data.first_sprite = self.0 as i32;
		user_data.last_sprite = self.1 as i32;
	}
}

impl<'a> Inspect<'a> for amethyst::renderer::SpriteRender {
	type UserData = &'a mut UserData;
	fn inspect(&mut self, entity: Entity, ui: &imgui::Ui<'_>, user_data: Self::UserData) {
		let mut sprite_number = self.sprite_number as i32;
		ui.slider_int(imgui::im_str!("# sprite##sprite_render{}{}", entity.id(), entity.gen().id()), &mut sprite_number, user_data.first_sprite, user_data.last_sprite).build();
		self.sprite_number = sprite_number as usize;
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

![screenshot](https://raw.githubusercontent.com/awpteamoose/amethyst-inspector/master/screenshot.png)
