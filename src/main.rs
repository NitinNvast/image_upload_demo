use dioxus::prelude::*;

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
        // document::Stylesheet { href: TAILWIND_CSS }
        style { "{include_str!(\"../assets/tailwind.css\")}" }

        ImageUploader {}
        ImageUploader21 {}
        ImageUploader22 {}
    }
}
