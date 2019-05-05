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
	no_default: Option<bool>,
}

#[derive(Debug, FromField)]
#[darling(attributes(inspect))]
struct FieldArgs {
	#[darling(default)]
	null_to: Option<syn::Lit>,
	#[darling(default)]
	speed: Option<f32>,
	#[darling(default)]
	skip: Option<bool>,
}

#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let no_default = StructArgs::from_derive_input(&input).unwrap().no_default.unwrap_or(false);

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

			const CAN_ADD: bool = #can_add;

			#inspect
			fn add((lazy, ..): &mut Self::SystemData, entity: ::amethyst::ecs::Entity) { #add }
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
						let skip = args.skip.unwrap_or(false);
						if skip { return quote!(); };

						let null_to = args.null_to.map(|x| quote!(#x)).unwrap_or(quote!(0.));
						let speed = args.speed.unwrap_or(0.1);
						let name = &f.ident;
						let ty = &f.ty;
						quote_spanned!{f.span()=>
							changed = <#ty as ::amethyst_inspector::InspectControl>::control(&mut new_me.#name, #null_to, #speed, ::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)), ui) || changed;
						}
					});
					let extra_data = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip.unwrap_or(false);
						if skip { return quote!(); };

						let ty = &f.ty;
						quote!{ <#ty as ::amethyst_inspector::InspectControl<'a>>::SystemData, }
					});
					let extra_data_members = fields.named.iter().map(|f| {
						let args = FieldArgs::from_field(&f).unwrap();
						let skip = args.skip.unwrap_or(false);
						if skip { return quote!() };

						let storage = format!("storage_{}", f.ident.as_ref().unwrap());
						let varname = syn::Ident::new(&storage, f.span());
						quote!{#varname, }
					});
					(quote! {
						fn inspect((lazy, storage, #(#extra_data_members)*): &mut Self::SystemData, entity: ::amethyst::ecs::Entity, ui: &::amethyst_imgui::imgui::Ui<'_>) {
							let me = if let Some(x) = storage.get(entity) { x } else { return; };
							let mut new_me = me.clone();
							let mut changed = false;
							ui.push_id(::amethyst_imgui::imgui::im_str!("{}", stringify!(#name)));

							#(#inspect_fields)*

							if changed {
								lazy.insert(entity, new_me);
							}
							ui.pop_id();
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
