use std::hash::{DefaultHasher, Hash, Hasher};

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(FromDeriveInput)]
#[darling(attributes(page))]
pub struct ComponentOps {
    js_path: Option<String>,
}

pub fn component(input: TokenStream) -> TokenStream {
    let mut hasher = DefaultHasher::new();
    input.to_string().hash(&mut hasher);
    let hash = hasher.finish();

    // Parse the string representation
    let ast = parse_macro_input!(input as DeriveInput);
    let ComponentOps { js_path } = ComponentOps::from_derive_input(&ast).unwrap();

    let ident = ast.ident;

    let lower_ident = ident.to_string().to_ascii_lowercase();
    let tag = format!("component-{}", lower_ident);

    let client_insert = if let Some(js_path) = js_path {
        let hydrate_str = format!("hydrate_{lower_ident}");
        quote! {
            entity.insert(elementary_rs_lib::js_path::JSPath(#js_path.to_string()));
            entity.insert(elementary_rs_lib::hydration_fn_name::HydrationFnName(#hydrate_str.to_string()));
        }
    } else {
        quote! {}
    };

    quote! {
        impl elementary_rs_lib::component::Component for #ident {
            fn build_entity(self) -> bevy_ecs::entity::Entity {
                let mut world = elementary_rs_lib::world::WORLD.write().unwrap();

                let mut entity = world.spawn((
                  elementary_rs_lib::node::AnyView(std::sync::Arc::new(self)),
                  elementary_rs_lib::selector::Selector::Id(#hash.to_string()),
                  elementary_rs_lib::tag::Tag(#tag.to_string())
                ));

                #client_insert

                entity.id()
            }
        }
    }
    .into()
}
