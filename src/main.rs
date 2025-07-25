use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus::LaunchBuilder;

const TAILWIND_CSS: Asset = asset!("assets/tailwind.css");

mod image_upload;
use crate::image_upload::ImageUploader;
mod img_upload_21;
use crate::img_upload_21::ImageUploader21;
mod img_upload_22;
use crate::img_upload_22::ImageUploader22;
mod chrome_style_navbar;
use crate::chrome_style_navbar::ChromeStyleNavbar;
mod img_upload_29;
use crate::img_upload_29::ImageUploader29;
mod img_upload_27;
use crate::img_upload_27::ImageUploader27;
mod img_upload_30;
use crate::img_upload_30::ImageUploader30;
mod opencv_img_12;
use crate::opencv_img_12::RFD_Image_Upload;

mod img_upload_28;
use crate::img_upload_28::ImageUploader28;

mod img_upload_31;
use crate::img_upload_31::ImageUploader31;

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            Config::default()
                .with_window(WindowBuilder::new().with_title("My App"))
                .with_menu(None),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        // document::Stylesheet { href: TAILWIND_CSS }
        style { "{include_str!(\"../assets/main.css\")}" }
        style { "{include_str!(\"../assets/tailwind.css\")}" }


            // ImageUploader {}
            // ImageUploader21 {}
            // ImageUploader22 {}
            // ImageUploader29 {  }
            // ImageUploader28 {}
            // ChromeStyleNavbar {}
            // ImageUploader27 {}
            // ImageUploader30 {}
            // RFD_Image_Upload {}
            ImageUploader31 {}


    }
}
