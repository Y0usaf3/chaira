use leptos::prelude::*;
use models::Value;
use models::ValueType;
use models::{FieldConfig, NumberConfig, RecordId, TextConfig};

fn value_to_string(value: &Value) -> String {
    match value {
        Value::SingleLine(v) => v.value().to_string(),
        Value::LongText(v) => v.value().to_string(),
        Value::Email(v) => v.value().to_string(),
        Value::URL(v) => v.value().to_string(),
        Value::Phone(v) => v.value().to_string(),
        Value::Number(v) => v.value().to_string(),
        Value::Decimal(v) => v.value().0.to_string(),
        Value::Currency(v) => v.value().0.to_string(),
        Value::Percent(v) => v.value().to_string(),
        Value::Rating(v) => v.value().to_string(),
        Value::Date(v) => v.value().to_string(),
        _ => String::new(),
    }
}

fn validate_string(field_config: &FieldConfig, val: &str) -> bool {
    if val.is_empty() {
        return true;
    }
    match field_config {
        FieldConfig::Text(tc) => match tc {
            TextConfig::Email => val.contains('@') && val.contains('.'),
            TextConfig::URL => val.starts_with("http://") || val.starts_with("https://"),
            TextConfig::Phone => val
                .chars()
                .all(|c| c.is_ascii_digit() || "+- ()".contains(c)),
            _ => true,
        },
        FieldConfig::Number(_) => val.parse::<f64>().is_ok(),
        _ => true,
    }
}

fn filter_string(field_config: &FieldConfig, val: &str) -> String {
    match field_config {
        FieldConfig::Text(tc) => match tc {
            TextConfig::SingleLine { .. } => val.replace('\n', " ").replace('\r', ""),
            TextConfig::Phone => val
                .chars()
                .filter(|c| c.is_ascii_digit() || "+- ()".contains(*c))
                .collect(),
            _ => val.to_string(),
        },
        FieldConfig::Number(nc) => match nc {
            NumberConfig::Number { .. } => val.chars().filter(|c| c.is_ascii_digit()).collect(),
            NumberConfig::Decimal { .. } => val
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect(),
            _ => val.to_string(),
        },
        _ => val.to_string(),
    }
}

fn needs_filtered(field_config: &FieldConfig) -> bool {
    match field_config {
        FieldConfig::Text(tc) => {
            matches!(tc, TextConfig::SingleLine { .. } | TextConfig::Phone)
        }
        FieldConfig::Number(_) => true,
        _ => false,
    }
}

#[component]
pub fn Cell(
    field_config: FieldConfig,
    field_name: String,
    value: Value,
    on_change: Callback<(RecordId, String, String)>,
    record_id: RecordId,
) -> impl IntoView {
    let display = value_to_string(&value);
    let (val, set_val) = signal(display);
    let (has_error, set_has_error) = signal(false);
    let needs_f = needs_filtered(&field_config);

    if needs_f {
        let input_cb = {
            let fg = field_config.clone();
            let sv = set_val.clone();
            move |ev: leptos::ev::Event| {
                let raw = event_target_value(&ev);
                let filtered = filter_string(&fg, &raw);
                sv.set(filtered);
            }
        };
        let keydown_cb = {
            let rc = record_id.clone();
            let fnm = field_name.clone();
            let oc = on_change.clone();
            move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Enter" {
                    let v = val.get_untracked();
                    oc.run((rc.clone(), fnm.clone(), v));
                }
            }
        };
        let blur_cb = {
            let rc = record_id.clone();
            let fnm = field_name.clone();
            let oc = on_change.clone();
            move |_| {
                let v = val.get_untracked();
                oc.run((rc.clone(), fnm.clone(), v));
            }
        };
        view! {
            <div class="border-b border-r border-slate-200 min-h-[32px] flex items-center px-1">
                <input
                    type="text"
                    class="w-full bg-transparent outline-none text-sm px-1"
                    prop:value=Signal::derive(move || val.get())
                    on:input=input_cb
                    on:keydown=keydown_cb
                    on:blur=blur_cb
                />
            </div>
        }
        .into_any()
    } else {
        let input_cb = {
            let sv = set_val.clone();
            let he = set_has_error.clone();
            move |ev: leptos::ev::Event| {
                sv.set(event_target_value(&ev));
                he.set(false);
            }
        };
        let keydown_cb = {
            let rc = record_id.clone();
            let fnm = field_name.clone();
            let oc = on_change.clone();
            let fg = field_config.clone();
            let he = set_has_error.clone();
            move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Enter" {
                    let v = val.get_untracked();
                    let valid = validate_string(&fg, &v);
                    he.set(!valid);
                    if valid {
                        oc.run((rc.clone(), fnm.clone(), v));
                    }
                }
            }
        };
        let blur_cb = {
            let rc = record_id.clone();
            let fnm = field_name.clone();
            let oc = on_change.clone();
            let fg = field_config.clone();
            let he = set_has_error.clone();
            move |_| {
                let v = val.get_untracked();
                if !v.is_empty() {
                    let valid = validate_string(&fg, &v);
                    he.set(!valid);
                    if valid {
                        oc.run((rc.clone(), fnm.clone(), v));
                    }
                }
            }
        };
        view! {
            <div class="relative border-b border-r border-slate-200 min-h-[32px] flex items-center px-1">
                <input
                    type="text"
                    class="w-full bg-transparent outline-none text-sm px-1"
                    prop:value=Signal::derive(move || val.get())
                    on:input=input_cb
                    on:keydown=keydown_cb
                    on:blur=blur_cb
                />
                <div
                    class="absolute bottom-0 right-0 w-1.5 h-1.5 transition-opacity"
                    class:opacity-100=move || has_error.get()
                    class:opacity-0=move || !has_error.get()
                    style="background-color: #ef4444"
                ></div>
            </div>
        }.into_any()
    }
}
