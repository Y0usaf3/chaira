use leptos::prelude::*;

#[component]
pub fn PlusIcon<'a>(class: &'a str) -> impl IntoView {
    view! {
        <div class=class>
            <svg
                width="800px"
                height="800px"
                viewBox="0 0 16 16"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M9 1H7V7H1V9H7V15H9V9H15V7H9V1Z" fill="currentColor" />
            </svg>
        </div>
    }
}
