use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

mod image_upload;
use crate::image_upload::ImageUploader;
mod img_upload_21;
use crate::img_upload_21::ImageUploader21;
mod img_upload_22;
use crate::img_upload_22::ImageUploader22;
fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Stylesheet { href: TAILWIND_CSS }

        ImageUploader {}
        ImageUploader21 {}
        ImageUploader22 {}
    }
}
