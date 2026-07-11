use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use models::Field;

use crate::components::{Icon, IconType, icon::field_icon};

#[component]
pub fn Field(
    field: Field,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
    delete_field_thingy: Callback<models::FieldId>,
) -> impl IntoView {
    let icon_type = field_icon(&field.config);

    let handle_ref = NodeRef::<leptos::html::Div>::new();
    let (is_resizing, set_is_resizing) = signal(false);
    let start_x = RwSignal::new(0.0);
    let local_width = RwSignal::new(width.get_untracked());

    Effect::new(move |_| {
        if !is_resizing.get_untracked() {
            local_width.set(width.get());
        }
    });

    let field_id = field.id.unwrap();
    let field_id = field_id.clone();

    let on_pointerdown = move |ev: leptos::ev::PointerEvent| {
        ev.prevent_default();
        set_is_resizing.set(true);
        start_x.set(ev.client_x() as f64);
        let _ = handle_ref
            .get()
            .map(|el| el.set_pointer_capture(ev.pointer_id()));
    };

    let on_pointermove = move |ev: leptos::ev::PointerEvent| {
        if is_resizing.get_untracked() {
            let dx = (ev.client_x() as f64).round()
                - ((ev.client_x() as f64).round() % 2.0)
                - start_x.get();
            let current = local_width.get_untracked();
            let new_w = current + dx;
            local_width.set(new_w);
            if let Some(ref cb) = on_resize {
                cb.run(new_w);
            }
            start_x.set(ev.client_x() as f64);
        }
    };

    let on_pointerup = move |ev: leptos::ev::PointerEvent| {
        if is_resizing.get_untracked() {
            set_is_resizing.set(false);
            let _ = handle_ref
                .get()
                .map(|el| el.release_pointer_capture(ev.pointer_id()));
        }
    };

    let field_ref = NodeRef::<leptos::html::Div>::new();

    let (show, set_show) = signal(false);

    let _ = window_event_listener(leptos::ev::click, move |ev| {
        if show.get_untracked() {
            if let Some(el) = field_ref.get() {
                if let Some(target) = ev.target() {
                    if let Some(node) = target.dyn_ref::<web_sys::Node>() {
                        if !el.contains(Some(node)) {
                            set_show.set(false);
                        }
                    }
                }
            }
        }
    });

    view! {
        <div
            node_ref=field_ref
            class="min-w-[100px] pl-2 h-full flex items-center border-black select-none bg-slate-50 relative"
            style:width=move || format!("{}px", local_width.get())
            on:contextmenu=move |ev| {
                ev.prevent_default();
                set_show.set(true);
            }
        >
            <Icon
                icon_type=icon_type
                class="w-[16px] h-auto fill-black mr-2 shrink-0"
                fill="#000000"
                extern_class="mr-[5px]"
            />
            <p class="truncate utility">{field.name}</p>
            <div
                node_ref=handle_ref
                class="ml-auto w-[3px] h-full cursor-col-resize bg-white hover:bg-black shrink-0 touch-none self-stretch"
                on:pointerdown=on_pointerdown
                on:pointermove=on_pointermove
                on:pointerup=on_pointerup
            ></div>
            <Show when=move || show.get()>
                <ol
                    class="bg-white border-black border-[2px] z-[9999] overflow-visible"
                    style:position="absolute"
                    style:top="40px"
                    style:left="2px"
                >
                    <li class="p-2 text-sm">"Rename"</li>
                    <li
                        class="p-2 text-sm flex flex-row text-[#ef4444] items-end"
                        on:click=move |ev| delete_field_thingy.run(field_id.clone())
                    >
                        <Icon
                            icon_type=IconType::Trash
                            fill="#ef4444"
                            class="!w-[18px] h-auto"
                            extern_class="mr-[4px]"
                        />
                        <p class="leading-none">"Delete"</p>
                    </li>
                </ol>
            </Show>
        </div>
    }
}
