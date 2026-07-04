use leptos::prelude::*;

#[component]
pub fn Column(children: Children, #[prop(optional)] width: Signal<f64>) -> impl IntoView {
    view! {
        <div
            class="flex flex-col min-w-[100px] divide-y-[2px] divide-black"
            style:width=move || format!("{}px", width.get())
        >
            {children()}
        </div>
    }
}
