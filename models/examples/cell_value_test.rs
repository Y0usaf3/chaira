use models::kinds::*;
use models::*;
use ordered_float::OrderedFloat;
use surrealdb_types::Datetime;

fn main() {
    println!("=== Cell Value Tests ===\n");

    test_single_line_value();
    test_long_text_value();
    test_email_value();
    test_url_value();
    test_phone_value();
    test_number_value();
    test_decimal_value();
    test_currency_value();
    test_percent_value();
    test_rating_value();
    test_date_value();

    println!("\n✓ All cell value tests passed!");
}

fn assert_verify_ok<T: ValueType<TT> + ?Sized, TT: ?Sized>(value: &T, config: FieldConfig) {
    assert!(value.verify(config).is_ok());
}

fn assert_verify_wrong_type<T: ValueType<TT> + ?Sized, TT: ?Sized>(
    value: &T,
    config: FieldConfig,
) {
    assert!(matches!(value.verify(config), Err(ValueError::WrongType(_))));
}

// ---------------------------------------------------------------------------
// SingleLineValue
// ---------------------------------------------------------------------------
fn test_single_line_value() {
    let v = SingleLineValue::new(None, Some("hello world".into())).unwrap();
    assert_eq!(ValueType::<str>::value(&v), "hello world");

    let config =
        FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::Email));

    let small =
        FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 2 });
    assert!(matches!(v.verify(small), Err(ValueError::TextTooBig(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::Email));
    assert!(matches!(r, Ok(Value::Email(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::URL));
    assert!(matches!(r, Ok(Value::URL(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Err(ValueError::CantConvertTo(_))));

    let v2 = SingleLineValue::new(None, Some("42".into())).unwrap();
    let r = v2.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = v2.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let r = v2.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(r, Ok(Value::Percent(_))));

    let v3 = SingleLineValue::new(None, Some("5".into())).unwrap();
    let r = v3.convert_to(&FieldConfig::Number(NumberConfig::Rating { max: 10, icon_type: RatingIcon::Star, color: [255, 200, 0] }));
    assert!(matches!(r, Ok(Value::Rating(_))));

    let r = v2.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "USD".into(), precision: 2 }));
    assert!(matches!(r, Ok(Value::Currency(_))));

    let dt_config = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert!(matches!(v2.convert_to(&dt_config), Err(ValueError::CantConvertTo(_))));

    let date_input = SingleLineValue::new(None, Some("2024-01-15".into())).unwrap();
    assert!(matches!(date_input.convert_to(&dt_config), Ok(Value::Date(_))));

    let unsupported = FieldConfig::Custom(CustomConfig::Attachment);
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    assert!(SingleLineValue::new(None, None).is_err());
    assert!(SingleLineValue::new(None, Some("  ".into())).is_ok());

    println!("  ✓ SingleLineValue");
}

// ---------------------------------------------------------------------------
// LongTextValue
// ---------------------------------------------------------------------------
fn test_long_text_value() {
    let v = LongTextValue::new("hello long text".into(), false).unwrap();
    assert_eq!(ValueType::<str>::value(&v), "hello long text");

    let config = FieldConfig::Text(TextConfig::LongText { rich_text: false });
    assert_verify_ok(&v, config);

    let config_rich = FieldConfig::Text(TextConfig::LongText { rich_text: true });
    assert_verify_ok(&v, config_rich);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::Email));

    let rt = LongTextValue::new("hello **bold**".into(), true).unwrap();
    let config_no_rich = FieldConfig::Text(TextConfig::LongText { rich_text: false });
    assert!(matches!(rt.verify(config_no_rich), Err(ValueError::UnallowedRichType)));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let email_text = LongTextValue::new("contact me at test@example.com".into(), false).unwrap();
    let r = email_text.convert_to(&FieldConfig::Text(TextConfig::Email));
    assert!(matches!(r, Ok(Value::Email(_))));

    let url_text = LongTextValue::new("visit https://example.com".into(), false).unwrap();
    let r = url_text.convert_to(&FieldConfig::Text(TextConfig::URL));
    assert!(matches!(r, Ok(Value::URL(_))));

    let num_text = LongTextValue::new("42".into(), false).unwrap();
    let r = num_text.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = num_text.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let pct = num_text.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(pct, Ok(Value::Percent(_))));

    let rating_val = LongTextValue::new("5".into(), false).unwrap();
    let rating = rating_val.convert_to(&FieldConfig::Number(NumberConfig::Rating { max: 10, icon_type: RatingIcon::Star, color: [255, 200, 0] }));
    assert!(matches!(rating, Ok(Value::Rating(_))));

    let curr = num_text.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "USD".into(), precision: 2 }));
    assert!(matches!(curr, Ok(Value::Currency(_))));

    let dt = num_text.convert_to(&FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false }));
    assert!(matches!(dt, Err(ValueError::CantConvertTo(_))));

    assert!(LongTextValue::new("".into(), false).is_ok());

    println!("  ✓ LongTextValue");
}

// ---------------------------------------------------------------------------
// Email
// ---------------------------------------------------------------------------
fn test_email_value() {
    let v = Email::new("user@example.com".into()).unwrap();
    assert_eq!(ValueType::<str>::value(&v), "user@example.com");

    let config = FieldConfig::Text(TextConfig::Email);
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::URL));

    let invalid = Email::new("not-an-email".into());
    assert!(invalid.is_err());

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let unsupported = FieldConfig::Number(NumberConfig::Number { default: None });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ Email");
}

// ---------------------------------------------------------------------------
// UrlValue
// ---------------------------------------------------------------------------
fn test_url_value() {
    let v = UrlValue::new("https://example.com".into()).unwrap();
    assert_eq!(ValueType::<str>::value(&v), "https://example.com");

    let config = FieldConfig::Text(TextConfig::URL);
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::Phone));

    assert!(UrlValue::new("not-a-url".into()).is_err());

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 200 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let unsupported = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ UrlValue");
}

// ---------------------------------------------------------------------------
// PhoneValue
// ---------------------------------------------------------------------------
fn test_phone_value() {
    let v = PhoneValue::new("+14155552671".into(), None).unwrap();
    assert!(ValueType::<str>::value(&v).contains("+1415"));

    let config = FieldConfig::Text(TextConfig::Phone);
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::Email));

    assert!(PhoneValue::new("not-a-phone".into(), None).is_err());

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let unsupported = FieldConfig::Number(NumberConfig::Number { default: None });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ PhoneValue");
}

// ---------------------------------------------------------------------------
// NumberValue
// ---------------------------------------------------------------------------
fn test_number_value() {
    let v = NumberValue::new(Some(42), None).unwrap();
    assert_eq!(*ValueType::<isize>::value(&v), 42);

    let config = FieldConfig::Number(NumberConfig::Number { default: None });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));

    assert!(NumberValue::new(None, None).is_err());

    let default_v = NumberValue::new(None, Some(99)).unwrap();
    assert_eq!(*ValueType::<isize>::value(&default_v), 99);

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "USD".into(), precision: 2 }));
    assert!(matches!(r, Ok(Value::Currency(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(r, Ok(Value::Percent(_))));

    let unsupported = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ NumberValue");
}

// ---------------------------------------------------------------------------
// DecimalValue
// ---------------------------------------------------------------------------
fn test_decimal_value() {
    let v = DecimalValue::new(Some(3.14), None).unwrap();
    assert_eq!(*ValueType::<OrderedFloat<f64>>::value(&v), OrderedFloat::from(3.14));

    let config = FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Number(NumberConfig::Number { default: None }));

    assert!(DecimalValue::new(None, None).is_err());

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "EUR".into(), precision: 2 }));
    assert!(matches!(r, Ok(Value::Currency(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(r, Ok(Value::Percent(_))));

    let unsupported = FieldConfig::Text(TextConfig::Email);
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ DecimalValue");
}

// ---------------------------------------------------------------------------
// CurrencyValue
// ---------------------------------------------------------------------------
fn test_currency_value() {
    let v = CurrencyValue::new(OrderedFloat::from(99.99));
    // ValueType<OrderedFloatIThink> — access via .0 twice
    assert_eq!((ValueType::<OrderedFloatIThink>::value(&v).0).0, 99.99);

    let config = FieldConfig::Number(NumberConfig::Currency { currency: "USD".into(), precision: 2 });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(r, Ok(Value::Percent(_))));

    let unsupported = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ CurrencyValue");
}

// ---------------------------------------------------------------------------
// PercentValue
// ---------------------------------------------------------------------------
fn test_percent_value() {
    let v = PercentValue::new(75);
    assert_eq!(*ValueType::<i32>::value(&v), 75);

    let config = FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Number(NumberConfig::Number { default: None }));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "GBP".into(), precision: 2 }));
    assert!(matches!(r, Ok(Value::Currency(_))));

    let unsupported = FieldConfig::Text(TextConfig::Email);
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ PercentValue");
}

// ---------------------------------------------------------------------------
// RatingValue
// ---------------------------------------------------------------------------
fn test_rating_value() {
    let v = RatingValue::new(Some(4), 5).unwrap();
    assert_eq!(*ValueType::<u8>::value(&v), 4);

    let config = FieldConfig::Number(NumberConfig::Rating { max: 5, icon_type: RatingIcon::Star, color: [255, 200, 0] });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Number(NumberConfig::Number { default: None }));

    assert!(RatingValue::new(Some(6), 5).is_err());

    let exceed = FieldConfig::Number(NumberConfig::Rating { max: 3, icon_type: RatingIcon::Heart, color: [200, 0, 0] });
    assert!(matches!(v.verify(exceed), Err(ValueError::BiggerThanMax)));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::LongText { rich_text: false }));
    assert!(matches!(r, Ok(Value::LongText(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Number { default: None }));
    assert!(matches!(r, Ok(Value::Number(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 }));
    assert!(matches!(r, Ok(Value::Decimal(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Currency { currency: "USD".into(), precision: 2 }));
    assert!(matches!(r, Ok(Value::Currency(_))));

    let r = v.convert_to(&FieldConfig::Number(NumberConfig::Percent { precision: 0, show_bar: false }));
    assert!(matches!(r, Ok(Value::Percent(_))));

    let unsupported = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ RatingValue");
}

// ---------------------------------------------------------------------------
// DateValue
// ---------------------------------------------------------------------------
fn test_date_value() {
    let now: Datetime = chrono::Utc::now().into();
    let v = DateValue::new(now.clone());
    assert_eq!(*ValueType::<Datetime>::value(&v), now);

    let config = FieldConfig::Datetime(DatetimeConfig::Date { format: DateFormat::ISO, include_time: false });
    assert_verify_ok(&v, config);

    assert_verify_wrong_type(&v, FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));

    let r = v.convert_to(&FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 100 }));
    assert!(matches!(r, Ok(Value::SingleLine(_))));

    let unsupported = FieldConfig::Number(NumberConfig::Number { default: None });
    assert!(matches!(v.convert_to(&unsupported), Err(ValueError::WrongType(_))));

    println!("  ✓ DateValue");
}
