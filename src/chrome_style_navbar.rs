use dioxus::prelude::*;

#[component]
pub fn ChromeStyleNavbar() -> Element {
    let mut tabs = use_signal(|| {
        vec![Tab {
            id: 1,
            title: "Tab 1".to_string(),
            active: true,
        }]
    });

    let mut next_id = use_signal(|| 2);

    let add_tab = move |_| {
        tabs.write().iter_mut().for_each(|t| t.active = false);
        tabs.write().push(Tab {
            id: next_id(),
            title: format!("Tab {}", next_id()),
            active: true,
        });
        next_id += 1;
    };

    let mut activate_tab = move |id| {
        tabs.write().iter_mut().for_each(|t| t.active = t.id == id);
    };

    let mut close_tab = move |id| {
        let mut new_list: Vec<Tab> = tabs().into_iter().filter(|t| t.id != id).collect();

        if !new_list.iter().any(|t| t.active) {
            if let Some(first) = new_list.first_mut() {
                first.active = true;
            }
        }

        tabs.set(new_list);
    };

    let rendered_tabs = tabs().into_iter().map(|tab| {
        let class_name = if tab.active {
            "flex items-center px-4 py-1 rounded-t-md cursor-pointer text-sm bg-white border border-gray-300"
        } else {
            "flex items-center px-4 py-1 rounded-t-md cursor-pointer text-sm bg-gray-200 hover:bg-gray-300"
        };

        rsx! {
            div {
                key: "{tab.id}",
                class: "{class_name}",
                onclick: move |_| activate_tab(tab.id),
                "{tab.title}",
                button {
                    class: "ml-2 text-gray-500 hover:text-red-600",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        close_tab(tab.id);
                    },
                    "×"
                }
            }
        }
    });

    // let rendered_tabs = tabs().into_iter().map(|tab| {
    //     let (bg_class, border_class, text_class, z_class) = if tab.active {
    //         (
    //             "bg-white",
    //             "border-x border-t border-gray-300",
    //             "text-black",
    //             "z-10",
    //         )
    //     } else {
    //         (
    //             "bg-[#e0e0e0] hover:bg-[#d0d0d0]",
    //             "border border-transparent",
    //             "text-gray-600",
    //             "z-0",
    //         )
    //     };

    //     let class_name = format!(
    //         "relative px-4 py-1 flex items-center gap-2 text-sm rounded-t-md {bg_class} {border_class} {text_class} {z_class} -mb-px"
    //     );

    //     rsx! {
    //         div {
    //             key: "{tab.id}",
    //             class: "{class_name}",
    //             onclick: move |_| activate_tab(tab.id),
    //             "{tab.title}",
    //             button {
    //                 class: "ml-2 text-gray-400 hover:text-red-500",
    //                 onclick: move |evt| {
    //                     evt.stop_propagation();
    //                     close_tab(tab.id);
    //                 },
    //                 "×"
    //             }
    //         }
    //     }
    // });

    rsx! {


     // Background under all tabs (gray with white 'cut' under active tab)
    div {
        class: "relative px-2 bg-[#dfe3e8] border-b border-gray-300 flex items-end space-x-1 overflow-visible",

        // Render tabs
        {rendered_tabs},

        // Vertical Divider
        div { class: "border-l h-5 border-gray-400 mx-1" },

        // "+" Button
        button {
            class: "w-6 h-6 mb-[1px] bg-gray-200 hover:bg-gray-300 rounded-full text-gray-700 text-lg flex items-center justify-center",
            onclick: add_tab,
            "+"
        }
    }
    }
}

#[derive(Clone, PartialEq)]
struct Tab {
    id: usize,
    title: String,
    active: bool,
}
