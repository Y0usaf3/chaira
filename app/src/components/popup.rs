use leptos::prelude::*;

#[component]
pub fn Popup(
    show: Signal<bool>,
    set_show: WriteSignal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
                on:click=move |_| set_show.set(false)
            >
                <div
                    class="pixel-corners--wrapper p-7 w-[400px] bg-white shadow-sm"
                    on:click=|ev| ev.stop_propagation()
                >
                    {children()}
                </div>
            </div>
        </Show>
    }
}
