use leptos::prelude::*;

#[component]
pub fn Column(
    children: Children,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
) -> impl IntoView {
    let handle_ref = NodeRef::<leptos::html::Div>::new();
    let (is_resizing, set_is_resizing) = signal(false);
    let start_x = RwSignal::new(0.0);
    let local_width = RwSignal::new(width.get_untracked());

    Effect::new(move |_| {
        if !is_resizing.get_untracked() {
            local_width.set(width.get());
        }
    });

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
            let dx = ev.client_x() as f64 - start_x.get();
            let current = local_width.get_untracked();
            let new_w = (current + dx).max(60.0);
            let stepped = ((new_w / 2.0).round() * 2.0) as f64;
            local_width.set(stepped);
            if let Some(ref cb) = on_resize {
                cb.run(stepped);
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

    view! {
        <div
            class="flex flex-col shrink-0 relative"
            style:width=move || format!("{}px", local_width.get())
        >
            {children()}
            <div
                node_ref=handle_ref
                class="absolute top-0 right-0 w-[4px] h-full cursor-col-resize bg-black shrink-0 touch-none z-10"
                style:opacity=move || if is_resizing.get() { "1" } else { "0" }
                style:background-color=move || {
                    if is_resizing.get() { "#000000" } else { "transparent" }
                }
                on:pointerdown=on_pointerdown
                on:pointermove=on_pointermove
                on:pointerup=on_pointerup
            ></div>
        </div>
    }
}
