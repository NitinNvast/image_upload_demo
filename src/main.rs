use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

mod image_upload;
use crate::image_upload::ImageUploader;
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }

        // style { "{include_str!(\"../assets/tailwind.css\")}" }

        // h1 { class: "flex justify-center text-8xl bg-red-100", "Hello World!" }
        ImageUploader {}
    }
}
