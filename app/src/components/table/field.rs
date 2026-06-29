use leptos::prelude::*;
use models::Field;

use crate::components::{Icon, icon::field_icon};

#[component]
pub fn Field(field: Field) -> impl IntoView {
    let icon_type = field_icon(&field.config);
    view! {
        <div class="w-fit shrink-0 max-w-xs px-2 h-full flex items-center border-r-[2px] border-black">
            <Icon icon_type=icon_type class="w-[16px] h-auto fill-black mr-2" />
            <p>{field.name}</p>
        </div>
    }
}
