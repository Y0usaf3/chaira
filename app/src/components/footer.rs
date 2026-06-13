use chrono::{Datelike, Local};
use leptos::prelude::*;

pub fn Footer() -> impl IntoView {
    let year = Local::now().year();

    view! {
        <div class="relative w-full pt-10">
            <p class="absolute bottom-[6px] left-0 p-2 small-font text-[1.6rem]">
                {year}-{year + 1}
            </p>
            <div class="absolute bottom-0 left-0 h-[6px] w-full pride_gradient"></div>
        </div>
    }
}
