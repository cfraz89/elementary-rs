use crate::{
    component::Component,
    hydration_fn_name::HydrationFnName,
    js_path::JSPath,
    node::{construct_entity_view, NodeRef},
    world::WORLD,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use crate::selector::Selector;
        use crate::server_data::ServerData;

        pub trait Page: Component {
            async fn render(self) -> Result<String, serde_json::Error> {
                let serial_page = serde_json::to_string(&self)?;
                //written out by macro
                let entity = self.build_entity();
                let hydration_fn_name: Option<HydrationFnName>;
                let js_path: Option<JSPath>;
                {
                    let world = WORLD.read().unwrap();
                    let entity_ref = world.entity(entity);
                    js_path = entity_ref.get::<JSPath>().cloned();
                    hydration_fn_name = entity_ref.get::<HydrationFnName>().cloned();
                }
                construct_entity_view(&entity, None).await.expect("failed constructing view");
                let res: String;
                {
                    let world = WORLD.read().unwrap();
                    let entity_ref = world.entity(entity);
                    let rendered_node = entity_ref.get::<NodeRef>().expect("page has no node").render(&world).expect("Render didnt give any output!");
                    let selector = entity_ref.get::<Selector>().to_owned().expect("Entity needs a selector");
                    let selector_attr = match selector {
                                            Selector::Id(id) => format!("id=\"_{id}\""),
                                            Selector::Class(class) => format!("class=\"_{class}\""),
                                        };
                    let script = match (js_path, hydration_fn_name) {
                        (Some(JSPath(js_path)), Some(HydrationFnName(hydration_fn_name))) => {
                            let serial_server_data = ServerData::get_serial_server_data(&entity);
                            let server_data_string = serde_json::to_string(&serial_server_data)?;
                            Ok(format!("<script type=\"module\">import start, {{ {hydration_fn_name} }} from \"{js_path}\"; await start(); await {hydration_fn_name}({serial_page}, {server_data_string});</script>"))
                        }
                        _ => Ok("".to_string())
                    }?;
                    res = format!(
                        "<!doctype html><html><head></head><body {selector_attr}>{rendered_node}{script}</body></html>",

                    );
                }
                {
                    WORLD.write().unwrap().clear_all();
                }
                    println!("Cleared world");
                    Ok(res)
            }
        }

    } else {
            use gloo_utils::format::JsValueSerdeExt;
            use serde::de::Deserialize;
            use wasm_bindgen::JsValue;
            use crate::server_data::SerialServerData;
            pub trait Page: Component {
                async fn hydrate(
                    serial_page: JsValue,
                    serial_server_data: JsValue,
                ) -> Result<(), JsValue> {
                    let page: Self = serial_page
                        .into_serde()
                        .expect("Could not deserialize initial value!");
                    let serial_server_data: SerialServerData = serial_server_data
                        .into_serde()
                        .expect("Could not deserialize server data!");
                    construct_entity_view(&page.build_entity(), Some(serial_server_data)).await?;
                    Ok(())
                }
            }
    }
}
