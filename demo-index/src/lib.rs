mod my_h1;

use elementary_rs_lib::{
    node::{Node, NodeRef, View},
    page::Page,
};
use elementary_rs_macros::{view, Component};
use my_h1::MyH1;
use serde::{Deserialize, Serialize};

#[derive(Component, bevy_ecs::component::Component, Serialize, Deserialize)]
#[page(js_path = "./wasm/demo_index.js")]
pub struct IndexPage {
    pub x: i32,
}

impl Page for IndexPage {}

impl View for IndexPage {
    async fn build(&self) -> NodeRef {
        view!(
            <div>
                <MyH1>
                    {self.x * 10}
                </MyH1>
            </div>
        )
    }
}
