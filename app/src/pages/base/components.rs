use crate::base::{BaseTable, TableData, TableField};
use crate::components::ui::{
    button::Button,
    dialog::{
        Dialog, DialogBody, DialogClose, DialogContent, DialogCustomTrigger, DialogDescription,
        DialogFooter, DialogHeader, DialogTitle,
    },
    dropdown_menu::{
        DropdownMenu, DropdownMenuAction, DropdownMenuContent, DropdownMenuLabel,
        DropdownMenuTrigger,
    },
    input::Input,
    label::Label,
    sheet::{
        Sheet, SheetBody, SheetClose, SheetContent, SheetCustomTrigger, SheetDescription,
        SheetFooter, SheetHeader, SheetTitle,
    },
};
use icons::{
    AlignLeft, Calendar, Cpu, FolderCode, Globe, Hash, Link, List, Lock, Mail, Phone, Plus,
    Settings, Trash, Type, User,
};
use leptos::prelude::*;

#[component]
pub fn FieldIcon(config: charac::models::field::FieldConfig) -> impl IntoView {
    use charac::models::field::{FieldConfig, TextConfig};
    match config {
        FieldConfig::Text(t) => match t {
            TextConfig::SingleLine { .. } => view! { <Type class="size-4" /> }.into_any(),
            TextConfig::LongText { .. } => view! { <AlignLeft class="size-4" /> }.into_any(),
            TextConfig::Email => view! { <Mail class="size-4" /> }.into_any(),
            TextConfig::URL => view! { <Globe class="size-4" /> }.into_any(),
            TextConfig::Phone => view! { <Phone class="size-4" /> }.into_any(),
        },
        FieldConfig::Number(_) => view! { <Hash class="size-4" /> }.into_any(),
        FieldConfig::Select(_) => view! { <List class="size-4" /> }.into_any(),
        FieldConfig::Datetime(_) => view! { <Calendar class="size-4" /> }.into_any(),
        FieldConfig::Relation(_) => view! { <Link class="size-4" /> }.into_any(),
        FieldConfig::User(_) => view! { <User class="size-4" /> }.into_any(),
        FieldConfig::Computed(_) => view! { <Cpu class="size-4" /> }.into_any(),
        FieldConfig::Custom(_) => view! { <Settings class="size-4" /> }.into_any(),
    }
}

#[component]
pub fn EditableFieldHeader(
    field_id: String,
    field_name: Signal<String>,
    config: Signal<charac::models::field::FieldConfig>,
    rename_action: Action<(String, String), Result<(), ServerFnError>>,
) -> impl IntoView {
    let (is_editing, set_is_editing) = signal(false);
    let (edit_value, set_edit_value) = signal(field_name.get_untracked());
    let (display_name, set_display_name) = signal(field_name.get_untracked());

    // Sync with external changes
    Effect::new(move |_| {
        let name = field_name.get();
        set_display_name.set(name.clone());
        if !is_editing.get_untracked() {
            set_edit_value.set(name);
        }
    });

    let input_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if is_editing.get() {
            if let Some(input) = input_ref.get() {
                let _ = input.focus();
            }
        }
    });

    let save = {
        let field_id = field_id.clone();
        move || {
            let val = edit_value.get_untracked();
            if !val.is_empty() {
                set_is_editing.set(false);
                if val != display_name.get_untracked() {
                    rename_action.dispatch((field_id.clone(), val.clone()));
                    set_display_name.set(val);
                }
            } else {
                set_edit_value.set(display_name.get_untracked());
                set_is_editing.set(false);
            }
        }
    };

    view! {
        <div class="w-full h-full flex items-center px-4">
            {move || {
                let save_for_blur = save.clone();
                let save_for_keydown = save.clone();
                if is_editing.get() {
                    view! {
                        <div class="flex items-center gap-2 w-full">
                            <FieldIcon config=config.get() />
                            <input
                                node_ref=input_ref
                                type="text"
                                class="w-full bg-background border-b border-primary focus:outline-none font-bold"
                                value=move || edit_value.get()
                                on:input=move |ev| set_edit_value.set(event_target_value(&ev))
                                on:blur=move |_| save_for_blur()
                                on:click=move |ev| ev.stop_propagation()
                                on:keydown=move |ev| {
                                    ev.stop_propagation();
                                    match ev.key().as_str() {
                                        "Enter" => save_for_keydown(),
                                        "Escape" => {
                                            set_edit_value.set(display_name.get());
                                            set_is_editing.set(false);
                                        }
                                        _ => {}
                                    }
                                }
                            />
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div
                            class="flex items-center gap-2 w-full h-full cursor-pointer"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                set_is_editing.set(true);
                            }
                        >
                            <FieldIcon config=config.get() />
                            <span>{move || display_name.get()}</span>
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn InlineFieldCreator(
    create_action: Action<(String, Option<String>), Result<TableField, ServerFnError>>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (selected_kind, set_selected_kind) = signal("Text".to_string());
    let input_ref = NodeRef::<leptos::html::Input>::new();

    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    });

    let submit = move || {
        let val = name.get_untracked();
        let kind = selected_kind.get_untracked();
        if !val.trim().is_empty() {
            create_action.dispatch((val, Some(kind)));
        }
        on_cancel.run(());
    };

    let kind_icon = move || match selected_kind.get().as_str() {
        "Number" => view! { <Hash class="size-4" /> }.into_any(),
        "Email" => view! { <Mail class="size-4" /> }.into_any(),
        "URL" => view! { <Globe class="size-4" /> }.into_any(),
        "Phone" => view! { <Phone class="size-4" /> }.into_any(),
        "LongText" => view! { <AlignLeft class="size-4" /> }.into_any(),
        "Date" => view! { <Calendar class="size-4" /> }.into_any(),
        _ => view! { <Type class="size-4" /> }.into_any(),
    };

    view! {
        <div class="flex items-center gap-2 w-full px-4 h-full">
            <DropdownMenu>
                <DropdownMenuTrigger class="hover:bg-muted p-1 rounded transition-colors">
                    {kind_icon}
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                    <DropdownMenuLabel>"Select Type"</DropdownMenuLabel>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("Text".to_string())>
                        <Type class="size-4 mr-2" />
                        "Text"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("Number".to_string())>
                        <Hash class="size-4 mr-2" />
                        "Number"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("LongText".to_string())>
                        <AlignLeft class="size-4 mr-2" />
                        "Long Text"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("Email".to_string())>
                        <Mail class="size-4 mr-2" />
                        "Email"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("Phone".to_string())>
                        <Phone class="size-4 mr-2" />
                        "Phone"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("URL".to_string())>
                        <Globe class="size-4 mr-2" />
                        "URL"
                    </DropdownMenuAction>
                    <DropdownMenuAction on:click=move |_| set_selected_kind.set("Date".to_string())>
                        <Calendar class="size-4 mr-2" />
                        "Date"
                    </DropdownMenuAction>
                </DropdownMenuContent>
            </DropdownMenu>
            <input
                node_ref=input_ref
                type="text"
                class="w-full bg-transparent border-none focus:outline-none font-bold text-sm"
                placeholder="Field name..."
                prop:value=move || name.get()
                on:input=move |ev| set_name.set(event_target_value(&ev))
                on:keydown=move |ev| {
                    match ev.key().as_str() {
                        "Enter" => submit(),
                        "Escape" => on_cancel.run(()),
                        _ => {}
                    }
                }

            />
        </div>
    }
}

#[component]
pub fn CreateTableDialog(
    title: impl IntoView + 'static,
    create_action: Action<String, Result<BaseTable, ServerFnError>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());

    let submit = move || {
        let val = name.get();
        if !val.is_empty() {
            create_action.dispatch(val);
            name.set(String::new());
        }
    };

    view! {
        <Dialog>
            <DialogCustomTrigger>{title}</DialogCustomTrigger>
            <DialogContent class="sm:max-w-[500px]">
                <DialogBody>
                    <DialogHeader>
                        <DialogTitle>"Create a Table!"</DialogTitle>
                        <DialogDescription>"Give your table a name."</DialogDescription>
                    </DialogHeader>
                    <div class="flex flex-col gap-4 py-4">
                        <Label html_for="table-name">"Table Name"</Label>
                        <Input
                            id="table-name"
                            bind_value=name
                            on:keydown=move |ev| if ev.key() == "Enter" { submit() }
                        />
                    </div>
                    <DialogFooter>
                        <DialogClose class="w-full sm:w-fit">"Cancel"</DialogClose>
                        <Button class="w-full sm:w-fit" on:click=move |_| submit()>
                            "Create"
                        </Button>
                    </DialogFooter>
                </DialogBody>
            </DialogContent>
        </Dialog>
    }
}

#[component]
pub fn TableBox(table: BaseTable) -> impl IntoView {
    view! {
        <div
            class="p-4 border rounded-lg bg-card hover:bg-accent cursor-pointer transition-colors"
            on:click=move |_| {
                window().location().assign(format!("/base/{}", table.id.clone()).as_str()).unwrap()
            }
        >
            <span class="font-bold text-lg">{table.name}</span>
            <p class="text-xs text-muted-foreground">"ID: " {table.id.clone()}</p>
        </div>
    }
}

#[component]
pub fn CreateFieldDialog(
    title: impl IntoView + 'static,
    create_action: Action<(String, Option<String>), Result<TableField, ServerFnError>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());

    let submit = move || {
        let val = name.get();
        if !val.is_empty() {
            create_action.dispatch((val, None));
            name.set(String::new());
        }
    };

    view! {
        <Dialog>
            <DialogCustomTrigger>{title}</DialogCustomTrigger>
            <DialogContent class="sm:max-w-[500px]">
                <DialogBody>
                    <DialogHeader>
                        <DialogTitle>"Create a Field!"</DialogTitle>
                        <DialogDescription>"Give your field a name. You can change the type later."</DialogDescription>
                    </DialogHeader>
                    <div class="flex flex-col gap-4 py-4">
                        <Label html_for="field-name">"Field Name"</Label>
                        <Input
                            id="field-name"
                            bind_value=name
                            on:keydown=move |ev| if ev.key() == "Enter" { submit() }
                        />
                    </div>
                    <DialogFooter>
                        <DialogClose class="w-full sm:w-fit">"Cancel"</DialogClose>
                        <Button class="w-full sm:w-fit" on:click=move |_| submit()>
                            "Create"
                        </Button>
                    </DialogFooter>
                </DialogBody>
            </DialogContent>
        </Dialog>
    }
}

#[component]
pub fn RenameFieldDialog(
    title: impl IntoView + 'static,
    current_name: String,
    rename_action: Action<(String, String), Result<(), ServerFnError>>,
    field_id: String,
) -> impl IntoView {
    let name = RwSignal::new(current_name);
    let f_id = StoredValue::new(field_id);

    let submit = move || {
        let val = name.get();
        if !val.is_empty() {
            rename_action.dispatch((f_id.get_value(), val));
        }
    };

    view! {
        <Sheet>
            <SheetCustomTrigger>{title}</SheetCustomTrigger>
            <SheetContent class="sm:max-w-[400px]">
                <SheetBody>
                    <SheetHeader>
                        <SheetTitle>"Rename Field"</SheetTitle>
                        <SheetDescription>"Enter the new name for this field."</SheetDescription>
                    </SheetHeader>
                    <div class="flex flex-col gap-4 py-4">
                        <Label html_for="rename-field-name">"Field Name"</Label>
                        <Input
                            id="rename-field-name"
                            bind_value=name
                            on:keydown=move |ev| if ev.key() == "Enter" { submit() }
                        />
                    </div>
                    <SheetFooter>
                        <SheetClose class="w-full sm:w-fit">"Cancel"</SheetClose>
                        <Button class="w-full sm:w-fit" on:click=move |_| submit()>
                            "Rename"
                        </Button>
                    </SheetFooter>
                </SheetBody>
            </SheetContent>
        </Sheet>
    }
}
