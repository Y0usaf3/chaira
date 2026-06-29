use leptos::portal::Portal;
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
}

#[component]
pub fn Select(
    #[prop(into)] options: Vec<SelectOption>,
    getter: ReadSignal<String>,
    setter: WriteSignal<String>,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);
    let (top, set_top) = signal(0.0_f64);
    let (left, set_left) = signal(0.0_f64);
    let (width, set_width) = signal(0.0_f64);

    let btn_ref = NodeRef::<leptos::html::Button>::new();
    let btn_ref_for_toggle = btn_ref.clone();

    let default_value = options.first().map(|o| o.value.clone()).unwrap_or_default();
    if getter.get_untracked().is_empty() {
        setter.set(default_value);
    }

    let toggle = move |_| {
        if !is_open.get_untracked() {
            #[cfg(target_arch = "wasm32")]
            if let Some(el) = btn_ref_for_toggle.get() {
                let rect = el.get_bounding_client_rect();
                set_top.set(rect.bottom());
                set_left.set(rect.left());
                set_width.set(rect.width());
            }
        }
        set_is_open.update(|open| *open = !*open);
    };

    let options_closure = options.clone();
    let for_fn = {
        let options = options.clone();
        move || options.clone()
    };
    let selected_label = move || {
        let current = getter.get();
        options_closure
            .iter()
            .find(|o| o.value == current)
            .map(|o| o.label.clone())
            .unwrap_or_else(|| "Select an option...".to_string())
    };

    view! {
        <div class="relative w-full font-['Pixel']">
            <button
                type="button"
                node_ref=btn_ref
                on:click=toggle
                class="w-full p-4 bg-neutral-900 border-2 border-white text-white flex justify-between items-center cursor-pointer text-left focus:outline-none"
            >
                <span>{selected_label}</span>
                <span class="ml-2">{move || if is_open.get() { "▲" } else { "▼" }}</span>
            </button>

            <Portal>
                <div
                    class="bg-neutral-900 border-2 border-white z-[9999] overflow-visible"
                    style:position="fixed"
                    style:top=move || format!("{}px", top.get())
                    style:left=move || format!("{}px", left.get())
                    style:width=move || format!("{}px", width.get())
                    style:display=move || if is_open.get() { "block" } else { "none" }
                >
                    <For
                        each=for_fn.clone()
                        key=|opt| opt.value.clone()
                        children=move |opt| {
                            let val = opt.value.clone();
                            let is_selected = move || getter.get() == opt.value;
                            let why_the_fuck_do_we_need_that_many_clones = is_selected.clone();

                            view! {
                                <div
                                    on:click=move |_| {
                                        setter.set(val.clone());
                                        set_is_open.set(false);
                                    }
                                    class="px-4 py-3 text-white cursor-pointer transition-colors duration-150 hover:bg-white hover:text-black flex justify-between items-center"
                                    class:bg-neutral-700=why_the_fuck_do_we_need_that_many_clones
                                >
                                    <span>{opt.label}</span>
                                    {move || {
                                        is_selected()
                                            .then(|| view! { <span class="text-xs">"✓"</span> })
                                    }}
                                </div>
                            }
                        }
                    />
                </div>
            </Portal>
        </div>
    }
}
