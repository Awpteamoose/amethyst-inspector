extern crate proc_macro;
use proc_macro2::TokenStream;
use proc_quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Ident, DataStruct};
use darling::{FromField, FromDeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(inspect))]
struct StructArgs {
	#[darling(default)]
	no_default: bool,
}

#[derive(Debug, FromField, Default)]
#[darling(attributes(inspect), default)]
struct FieldArgs {
	null_to: Option<syn::Lit>,
	speed: Option<f32>,
	skip: bool,
	#[darling(multiple)]
	with_component: Vec<syn::Path>,
}

#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let no_default = StructArgs::from_derive_input(&input).unwrap().no_default;

	let name = input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let (inspect, extra_data) = inspect(&input.data, &name);
	let (can_add, add) = match (no_default, input.data) {
		(_, Data::Struct(DataStruct { fields: Fields::Unit, .. })) => (true, quote!(lazy.insert(entity, Self);)),
		(false, _) => (true, quote!(lazy.insert(entity, Self::default());)),
		(true, _) => (false, quote!({})),
	};

	let expanded = quote! {
		impl<'a> #impl_generics ::amethyst_inspector::Inspect<'a> for #name #ty_generics #where_clause {
			type SystemData = (
				::amethyst::ecs::Read<'a, ::amethyst::ecs::LazyUpdate>,
				::amethyst::ecs::ReadStorage<'a, Self>,
				#extra_data
			);

			#inspect
			fn add((lazy, ..): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { #add }
			fn can_add(_: &mut Self::SystemData, _: ::amethyst::ecs::Entity) -> bool { #can_add }
		}
	};

	proc_macro::TokenStream::from(expanded)
}

fn inspect(data: &Data, name: &Ident) -> (TokenStream, TokenStream) {
	match *data {
		Data::Struct(ref data) => {
			match data.fields {
				Fields::Named(ref fields) => {
					let inspect_fields = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!(); };

						let name = &f.ident;
						let ty = &f.ty;
						let storage = format!("systemdata_{}", f.ident.as_ref().unwrap());
						let varname = syn::Ident::new(&storage, f.span());

						if !args.with_component.is_empty() {
							return with_component_body(f.ident.as_ref().unwrap(), varname);
						}

						// TODO: more field attrs
						let null_to = args.null_to.map(|x| quote!(.null_to(#x))).unwrap_or(quote!());
						let speed = args.speed.map(|x| quote!(.speed(#x))).unwrap_or(quote!());

						quote!{
							<&mut #ty as ::amethyst_inspector::InspectControl>::control(&mut new_me.#name)
								.changed(&mut changed)
								.data(#varname)
								#null_to
								#speed
								.label(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)))
								.build();
						}
					});
					let extra_data = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!(); };

						if !args.with_component.is_empty() {
							let paths = args.with_component;
							return quote!((
								#(ReadStorage<'a, #paths>,)*
								ReadStorage<'a, ::amethyst::core::Named>,
								::amethyst::ecs::Entities<'a>,
							),);
						}

						let ty = &f.ty;
						quote!{ <&'a mut #ty as ::amethyst_inspector::InspectControl<'a, 'a>>::SystemData, }
					});
					let extra_data_members = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip;
						if skip { return quote!() };

						let storage = format!("systemdata_{}", f.ident.as_ref().unwrap());
						let varname = syn::Ident::new(&storage, f.span());
						quote!{#varname, }
					});
					(quote! {
						fn inspect((lazy, storage, #(#extra_data_members)*): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) {
							::amethyst_imgui::with(|ui| {
								let me = if let Some(x) = storage.get(entity) { x } else { return; };
								let mut new_me = me.clone();
								let mut changed = false;
								ui.push_id(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)));

								#(#inspect_fields)*

								if changed {
									lazy.insert(entity, new_me);
								}
								ui.pop_id();
							});
						}
					}, quote!{#(#extra_data)*})
				}
				Fields::Unit => { (quote!(), quote!()) },
				_ => unimplemented!(),
			}
		},
		_ => unimplemented!(),
	}
}

fn with_component_body(name: &syn::Ident, data: syn::Ident) -> TokenStream {
	quote! {{
		use ::amethyst_imgui::imgui;

		let data = #data;
		let mut current = 0;
		let list = ::std::iter::once(None).chain((&data.0, &data.2).join().map(|x| Some(x.1))).collect::<Vec<_>>();
		let mut items = Vec::<imgui::ImString>::new();
		for (i, &entity) in list.iter().enumerate() {
				if new_me.#name == entity { current = i as i32; }

				let label: String = if let Some(entity) = entity {
					if let Some(name) = data.1.get(entity) {
						name.name.to_string()
					} else {
						format!("Entity {}/{}", entity.id(), entity.gen().id())
					}
				} else {
					"None".into()
				};
				items.push(imgui::im_str!("{}", label).into());
		}
		changed = ui.combo(imgui::im_str!("{}", stringify!(#name)), &mut current, items.iter().map(::std::ops::Deref::deref).collect::<Vec<_>>().as_slice(), 10) || changed;
		new_me.#name = list[current as usize];
	}}
}
