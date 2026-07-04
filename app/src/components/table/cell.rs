use leptos::prelude::*;
use models::{
    DatetimeConfig, DecimalValue, Email, FieldConfig, FieldId, LongTextValue, NumberConfig,
    NumberValue, PercentValue, PhoneValue, RatingValue, RecordId, SingleLineValue, TextConfig,
    UrlValue, Value, ValueError, ValueType,
};

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

fn create_value(field_config: &FieldConfig, raw: &str) -> Result<Value, ValueError> {
    match field_config {
        FieldConfig::Text(tc) => match tc {
            TextConfig::SingleLine { .. } => {
                let v = SingleLineValue::new(None, Some(raw.to_owned()))?;
                v.verify(field_config.clone())?;
                Ok(Value::SingleLine(v))
            }
            TextConfig::LongText { rich_text } => {
                let v = LongTextValue::new(raw.to_owned(), *rich_text)?;
                v.verify(field_config.clone())?;
                Ok(Value::LongText(Box::new(v)))
            }
            TextConfig::Email => {
                let v = Email::new(raw.to_owned())?;
                v.verify(field_config.clone())?;
                Ok(Value::Email(v))
            }
            TextConfig::URL => {
                let v = UrlValue::new(raw.to_owned())?;
                v.verify(field_config.clone())?;
                Ok(Value::URL(v))
            }
            TextConfig::Phone => {
                let v = PhoneValue::new(raw.to_owned(), None)?;
                v.verify(field_config.clone())?;
                Ok(Value::Phone(v))
            }
        },
        FieldConfig::Number(nc) => match nc {
            NumberConfig::Number { .. } => {
                let parsed = raw
                    .trim()
                    .parse::<isize>()
                    .map_err(|_| ValueError::CantConvertTo("Number".to_string()))?;
                let v = NumberValue::new(Some(parsed), None)?;
                v.verify(field_config.clone())?;
                Ok(Value::Number(v))
            }
            NumberConfig::Decimal { .. } => {
                let parsed = raw
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ValueError::CantConvertTo("Decimal".to_string()))?;
                let v = DecimalValue::new(Some(parsed), None)?;
                v.verify(field_config.clone())?;
                Ok(Value::Decimal(v))
            }
            NumberConfig::Currency { .. } => {
                let sl = SingleLineValue::new(None, Some(raw.to_owned()))?;
                sl.convert_to(field_config)
            }
            NumberConfig::Percent { .. } => {
                let cleaned = raw.replace('%', " ");
                let parsed = cleaned
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| ValueError::CantConvertTo("Percent".to_string()))?;
                let v = PercentValue::new(parsed);
                v.verify(field_config.clone())?;
                Ok(Value::Percent(v))
            }
            NumberConfig::Rating { max, .. } => {
                let parsed = raw
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ValueError::CantConvertTo("Rating".to_string()))?;
                let v = RatingValue::new(Some(parsed), *max as u8)?;
                v.verify(field_config.clone())?;
                Ok(Value::Rating(v))
            }
        },
        FieldConfig::Datetime(DatetimeConfig::Date { .. }) => {
            let sl = SingleLineValue::new(None, Some(raw.to_owned()))?;
            sl.convert_to(field_config)
        }
        _ => Err(ValueError::WrongType("unsupported field type".to_string())),
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
    field_name: FieldId,
    value: Value,
    on_change: Callback<(RecordId, FieldId, Value)>,
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
            let he = set_has_error.clone();
            move |ev: leptos::ev::Event| {
                let raw = event_target_value(&ev);
                let filtered = filter_string(&fg, &raw);
                sv.set(filtered);
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
                    match create_value(&fg, &v) {
                        Ok(value) => {
                            he.set(false);
                            oc.run((rc.clone(), fnm.clone(), value));
                        }
                        Err(_) => he.set(true),
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
                    match create_value(&fg, &v) {
                        Ok(value) => {
                            he.set(false);
                            oc.run((rc.clone(), fnm.clone(), value));
                        }
                        Err(_) => he.set(true),
                    }
                }
            }
        };
        view! {
            <div class="relative min-h-[32px] flex items-center px-1">
                <input
                    type="text"
                    class="w-full bg-transparent outline-none text-sm text-black px-1"
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
                    match create_value(&fg, &v) {
                        Ok(value) => {
                            he.set(false);
                            oc.run((rc.clone(), fnm.clone(), value));
                        }
                        Err(_) => he.set(true),
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
                    match create_value(&fg, &v) {
                        Ok(value) => {
                            he.set(false);
                            oc.run((rc.clone(), fnm.clone(), value));
                        }
                        Err(_) => he.set(true),
                    }
                }
            }
        };
        view! {
            <div class="relative min-h-[32px] flex items-center px-1">
                <input
                    type="text"
                    class="w-full bg-transparent outline-none text-sm text-black px-1"
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
        }
        .into_any()
    }
}
