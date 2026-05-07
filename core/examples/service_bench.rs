use charac::db::{DB, init};
use charac::models::field::kinds::*;
use charac::models::record::cell::*;
use charac::models::*;
use charac::service::approved;
use charac::service::base::BaseService;
use charac::service::crypter::{decrypt_token, encrypt_token};
use charac::service::table::{PaginationParams, TableService};
use charac::service::user::{AuthMethod, Session, UserService};
use std::collections::HashMap;
use std::time::Instant;
use surrealdb::types::ToSql;

macro_rules! bench_async {
    ($name:expr, $iterations:expr, $block:expr) => {
        let start = Instant::now();
        for _ in 0..$iterations {
            $block.await.expect("Benchmark failed");
        }
        let duration = start.elapsed();
        println!(
            "Benchmark {:<30} | Iterations: {:<6} | Total: {:<12?} | Avg: {:?}",
            $name,
            $iterations,
            duration,
            duration / $iterations
        );
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    init().await;

    test_crypter().await?;
    let (user_service, user_id) = test_user_service().await?;
    let (base_service, base_id) = test_base_service(&user_service, &user_id).await?;
    test_api_tokens(&user_service).await?;

    test_table_service(&base_service, &user_id, &base_id).await?;
    test_misc().await?;

    run_stress_test(&base_service, &user_id, &base_id).await?;

    println!("\n--- Iterative Benchmarks ---");
    run_benchmarks().await?;

    Ok(())
}

async fn run_stress_test(
    base_service: &BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n>>> STARTING SHITTON OF CHANGES STRESS TEST <<<");
    let table = base_service
        .create_table("mega_stress_table".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);
    let table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // 1. Add fields (shitton of fields)
    let field_definitions = vec![
        (
            "Name",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "Bio",
            FieldConfig::Text(TextConfig::LongText { rich_text: false }),
        ),
        ("Email", FieldConfig::Text(TextConfig::Email)),
        (
            "Age",
            FieldConfig::Number(NumberConfig::Number { default: None }),
        ),
        (
            "Salary",
            FieldConfig::Number(NumberConfig::Currency {
                currency: "USD".to_string(),
                precision: 2,
            }),
        ),
        (
            "Growth",
            FieldConfig::Number(NumberConfig::Percent {
                precision: 2,
                show_bar: true,
            }),
        ),
        (
            "Score",
            FieldConfig::Number(NumberConfig::Decimal {
                default: None,
                precision: 2,
            }),
        ),
        (
            "Rating",
            FieldConfig::Number(NumberConfig::Rating {
                max: 5,
                icon_type: RatingIcon::Star,
                color: [255, 255, 0],
            }),
        ),
    ];

    let mut field_names = Vec::new();
    let start_fields = Instant::now();
    for (name, config) in field_definitions {
        let insert = InsertField::new(name.to_string(), config, false, true, false);
        table_service.create_field(insert).await?;
        field_names.push(name.to_string());
    }
    let duration_fields = start_fields.elapsed();
    println!(
        "Added {} fields in {:?}",
        field_names.len(),
        duration_fields
    );

    // 2. Insert records (100 records)
    let num_records = 100000;
    let mut record_ids = Vec::new();
    let start_inserts = Instant::now();
    for i in 0..num_records {
        let mut cells = HashMap::new();
        cells.insert(
            "Name".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("User {}", i)),
            )?)),
        );
        cells.insert("Bio".to_string(), CellValue::new(Value::LongText(Box::new(LongTextValue::new(format!("User {} bio is quite long and contains interesting facts about this stress test user.", i), false)?))));
        cells.insert(
            "Email".to_string(),
            CellValue::new(Value::Email(Email::new(format!("user{}@example.com", i))?)),
        );
        cells.insert(
            "Age".to_string(),
            CellValue::new(Value::Number(NumberValue::new(Some(20 + (i % 30)), None)?)),
        );
        cells.insert(
            "Salary".to_string(),
            CellValue::new(Value::Currency(CurrencyValue::new(
                (1000 + i * 100) as i64,
                iso_currency::Currency::USD.symbol(),
            ))),
        );
        cells.insert(
            "Growth".to_string(),
            CellValue::new(Value::Percent(PercentValue::new((i % 100) as i32))),
        );
        cells.insert(
            "Score".to_string(),
            CellValue::new(Value::Decimal(DecimalValue::new(
                Some(i as f64 * 1.5),
                None,
            )?)),
        );
        cells.insert(
            "Rating".to_string(),
            CellValue::new(Value::Rating(RatingValue::new(Some((i % 6) as u8), 5)?)),
        );

        let insert = InsertRecord::new(table_id.clone(), cells);
        let record = table_service.create_record(insert).await?;
        record_ids.push(RecordId(record.id.unwrap().0));
    }
    let duration_inserts = start_inserts.elapsed();
    println!("Inserted {} records in {:?}", num_records, duration_inserts);

    // 3. Update records (Update ALL of them)
    let start_updates = Instant::now();
    for (i, id) in record_ids.iter().enumerate() {
        let mut changes = Vec::new();
        changes.push((
            "Name".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("Updated User {}", i)),
            )?)),
        ));
        changes.push((
            "Score".to_string(),
            CellValue::new(Value::Decimal(DecimalValue::new(
                Some(i as f64 * 2.0),
                None,
            )?)),
        ));
        let patch = RecordPatch::new(Some(changes));
        table_service.update_record(id.clone(), patch).await?;
    }
    let duration_updates = start_updates.elapsed();
    println!("Updated {} records in {:?}", num_records, duration_updates);

    // 4. List and Display
    let start_list = Instant::now();
    let records = table_service
        .list_records(PaginationParams {
            offset: None,
            limit: Some(1000),
        })
        .await?;
    let duration_list = start_list.elapsed();

    println!("\n>>> STRESS TEST FINAL RESULT (Sample of 300 records) <<<");
    print_records_table(&records);

    print_bench_table(
        "Mega Stress Test Summary",
        vec![
            ("Fields Added".to_string(), field_names.len().to_string()),
            ("Records Inserted".to_string(), num_records.to_string()),
            ("Records Updated".to_string(), num_records.to_string()),
            (
                "Avg Insert Latency".to_string(),
                format!("{:?}", duration_inserts / num_records as u32),
            ),
            (
                "Avg Update Latency".to_string(),
                format!("{:?}", duration_updates / num_records as u32),
            ),
            (
                "List Latency (1000 recs)".to_string(),
                format!("{:?}", duration_list),
            ),
        ],
        start_fields.elapsed(),
    );

    Ok(())
}

async fn test_crypter() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let token = "my-secret-token";
    let encrypted = encrypt_token(token).await?;
    let decrypted = decrypt_token(encrypted.clone()).await?;
    let duration = start.elapsed();

    assert_eq!(token, decrypted);

    print_bench_table(
        "Crypter: Encrypt/Decrypt",
        vec![
            ("Input".to_string(), token.to_string()),
            ("Encrypted".to_string(), encrypted),
            ("Decrypted".to_string(), decrypted),
        ],
        duration,
    );
    Ok(())
}

fn print_bench_table(title: &str, fields: Vec<(String, String)>, duration: std::time::Duration) {
    let name_width = fields
        .iter()
        .map(|(k, _)| k.len())
        .max()
        .unwrap_or(0)
        .max(5); // "Field"
    let val_width = fields
        .iter()
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(0)
        .max(5); // "Value"

    println!("\n[ {} ]", title);
    println!(
        "╭{}┬{}╮",
        "─".repeat(name_width + 2),
        "─".repeat(val_width + 2)
    );
    println!(
        "│ {:<nw$} │ {:<vw$} │",
        "Field",
        "Value",
        nw = name_width,
        vw = val_width
    );
    println!(
        "├{}┼{}┤",
        "─".repeat(name_width + 2),
        "─".repeat(val_width + 2)
    );
    for (k, v) in fields {
        println!(
            "│ {:<nw$} │ {:<vw$} │",
            k,
            v,
            nw = name_width,
            vw = val_width
        );
    }
    println!(
        "╰{}┴{}╯",
        "─".repeat(name_width + 2),
        "─".repeat(val_width + 2)
    );
    println!("Benchmark: {:?}\n", duration);
}

async fn test_user_service() -> Result<(UserService, UserId), Box<dyn std::error::Error>> {
    let start = Instant::now();
    // Manually create a user for testing
    let user: User = DB
        .query("CREATE user SET first_name = 'Test', last_name = 'User', email = 'test@example.com', role = 'admin' RETURN AFTER")
        .await?
        .take::<Option<User>>(0)?
        .unwrap();
    let duration = start.elapsed();
    let user_id = UserId(user.id.as_ref().unwrap().0.clone());

    print_bench_table(
        "UserService: Manual User Creation",
        vec![
            ("ID".to_string(), user_id.0.to_sql_pretty()),
            ("Email".to_string(), user.email.clone()),
            ("Role".to_string(), format!("{:?}", user.role())),
        ],
        duration,
    );

    // Test UserService::login with session (bypass HCAUTH for now)
    let start = Instant::now();
    let token = "test-session-token";
    let ip = "127.0.0.1".to_string();
    let agent = "TestAgent".to_string();

    DB.query("CREATE session SET user = $user, token = $tokenn, ip = $ip, user_agent = $agent, expires_at = time::now() + 1d")
        .bind(("user", user_id.clone()))
        .bind(("tokenn", token))
        .bind(("ip", ip.clone()))
        .bind(("agent", agent.clone()))
        .await?;

    let user_service = UserService::login(AuthMethod::Session(Session {
        token: token.to_string(),
        ip,
        agent,
    }))
    .await?;
    let duration = start.elapsed();

    assert_eq!(user_service.user.email, "test@example.com");
    print_bench_table(
        "UserService: Login",
        vec![
            ("Email".to_string(), user_service.user.email.clone()),
            ("AuthMethod".to_string(), "Session".to_string()),
        ],
        duration,
    );

    // Test is_admin
    let start = Instant::now();
    let is_admin = user_service.is_admin().await?;
    let duration = start.elapsed();
    assert!(is_admin);
    print_bench_table(
        "UserService: is_admin",
        vec![("Is Admin".to_string(), is_admin.to_string())],
        duration,
    );

    // Test create_base
    let start = Instant::now();
    let base = user_service.create_base("TestBase".to_string()).await?;
    let duration = start.elapsed();
    assert_eq!(base.name, "TestBase");
    print_bench_table(
        "UserService: create_base",
        vec![
            ("Base Name".to_string(), base.name.clone()),
            (
                "Base ID".to_string(),
                base.id.as_ref().unwrap().0.to_sql_pretty(),
            ),
        ],
        duration,
    );

    Ok((user_service, user_id))
}

async fn test_base_service(
    user_service: &UserService,
    user_id: &UserId,
) -> Result<(BaseService, BaseId), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let bases = user_service.list_bases().await?;
    let base_id = BaseId(bases[0].id.as_ref().unwrap().0.clone());

    // Test BaseService::new
    let base_service = BaseService::new(base_id.clone(), user_id.clone()).await?;
    let duration = start.elapsed();
    assert_eq!(base_service.id().0, base_id.0);
    print_bench_table(
        "BaseService: new",
        vec![("Base ID".to_string(), base_service.id().0.to_sql_pretty())],
        duration,
    );

    // Test create_table
    let start = Instant::now();
    let table = base_service.create_table("test_table".to_string()).await?;
    let duration = start.elapsed();
    assert_eq!(table.name, "test_table");
    print_bench_table(
        "BaseService: create_table",
        vec![
            ("Table Name".to_string(), table.name.clone()),
            (
                "Table ID".to_string(),
                table.id.as_ref().unwrap().0.to_sql_pretty(),
            ),
        ],
        duration,
    );

    let table_id = TableId(table.id.unwrap().0);

    // Test open_table
    let start = Instant::now();
    let _table_service = base_service.open_table(table_id.clone()).await?;
    let duration = start.elapsed();
    print_bench_table(
        "BaseService: open_table",
        vec![("Table ID".to_string(), table_id.0.to_sql_pretty())],
        duration,
    );

    Ok((base_service, base_id))
}

async fn test_table_service(
    base_service: &BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    let table = base_service
        .create_table("table_for_tests".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);

    let table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // Test create_field
    let start = Instant::now();
    let config = FieldConfig::Text(TextConfig::SingleLine {
        default: None,
        max_length: 255,
    });
    let insert_field = InsertField::new("test_field".to_string(), config, false, true, false);

    let field = table_service.create_field(insert_field).await?;
    let duration = start.elapsed();
    assert_eq!(field.name, "test_field");
    print_bench_table(
        "TableService: create_field",
        vec![
            ("Field Name".to_string(), field.name.clone()),
            (
                "Field ID".to_string(),
                field.id.as_ref().unwrap().0.to_sql_pretty(),
            ),
        ],
        duration,
    );

    let field_id = FieldId(field.id.clone().unwrap().0);

    // Test get_field_config
    let start = Instant::now();
    let config_fr = table_service.get_field_config(field_id.clone()).await?;
    let duration = start.elapsed();
    assert_eq!(config_fr.name, "test_field");
    print_bench_table(
        "TableService: get_field_config",
        vec![("Field Name".to_string(), config_fr.name.clone())],
        duration,
    );

    // Test create_record
    let start = Instant::now();
    let mut cells = HashMap::new();
    let val = Value::SingleLine(SingleLineValue::new(None, Some("test value".to_string()))?);
    cells.insert("test_field".to_string(), CellValue::new(val));
    let insert_record = InsertRecord::new(table_id.clone(), cells);
    let record = table_service.create_record(insert_record).await?;
    let duration = start.elapsed();
    print_bench_table(
        "TableService: create_record",
        vec![(
            "Record ID".to_string(),
            record.id.as_ref().unwrap().0.to_sql_pretty(),
        )],
        duration,
    );

    let record_id = RecordId(record.id.clone().unwrap().0);

    // Test get_record
    let start = Instant::now();
    let fetched_record = table_service.get_record(record_id.clone()).await?;
    let duration = start.elapsed();
    let val_str =
        if let Value::SingleLine(ref slv) = fetched_record.cells.get("test_field").unwrap().value {
            slv.value().to_string()
        } else {
            panic!("Wrong value type");
        };
    assert_eq!(val_str, "test value");
    print_bench_table(
        "TableService: get_record",
        vec![
            ("Record ID".to_string(), record_id.0.to_sql_pretty()),
            ("test_field".to_string(), val_str),
        ],
        duration,
    );

    // Test update_record
    let start = Instant::now();
    let mut changed_cells = Vec::new();
    let val_updated = Value::SingleLine(SingleLineValue::new(
        None,
        Some("updated value".to_string()),
    )?);
    changed_cells.push(("test_field".to_string(), CellValue::new(val_updated)));
    let patch = RecordPatch::new(Some(changed_cells));
    let updated_record = table_service
        .update_record(record_id.clone(), patch)
        .await?;
    let duration = start.elapsed();
    let val_updated_str =
        if let Value::SingleLine(ref slv) = updated_record.cells.get("test_field").unwrap().value {
            slv.value().to_string()
        } else {
            panic!("Wrong value type");
        };
    assert_eq!(val_updated_str, "updated value");
    print_bench_table(
        "TableService: update_record",
        vec![
            ("Record ID".to_string(), record_id.0.to_sql_pretty()),
            ("test_field (updated)".to_string(), val_updated_str),
        ],
        duration,
    );

    // Test list_records with Benchmarking
    let start = Instant::now();
    let records = table_service
        .list_records(PaginationParams {
            offset: Some(0),
            limit: Some(10),
        })
        .await?;
    let duration = start.elapsed();

    print_records_table(&records);
    println!("Benchmark (list_records): {:?}\n", duration);

    assert!(!records.is_empty());

    // Test check_migration
    let start = Instant::now();
    let target_config = FieldConfig::Number(NumberConfig::Number { default: None });
    let report = table_service
        .check_migration(field_id.clone(), target_config)
        .await?;
    let duration = start.elapsed();
    assert!(report.affected_records > 0);
    print_bench_table(
        "TableService: check_migration",
        vec![(
            "Affected Records".to_string(),
            report.affected_records.to_string(),
        )],
        duration,
    );

    Ok(())
}

fn print_records_table(records: &[Record]) {
    if records.is_empty() {
        println!("No records found.");
        return;
    }

    // Get all unique field names as headers
    let mut headers: Vec<String> = records
        .iter()
        .flat_map(|r| r.cells.keys().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    headers.sort();

    // Calculate column widths
    let mut widths: HashMap<String, usize> = headers.iter().map(|h| (h.clone(), h.len())).collect();

    for record in records {
        for header in &headers {
            if let Some(cell) = record.cells.get(header) {
                let val_str = cell.value.to_string();
                let entry = widths.get_mut(header).unwrap();
                *entry = (*entry).max(val_str.len());
            }
        }
    }

    // Top border
    print!("╭");
    for (i, header) in headers.iter().enumerate() {
        print!("{}", "─".repeat(widths[header] + 2));
        if i < headers.len() - 1 {
            print!("┬");
        }
    }
    println!("╮");

    // Header row
    print!("│");
    for (i, header) in headers.iter().enumerate() {
        print!(" {:<width$} ", header, width = widths[header]);
        print!("│");
    }
    println!();

    // Divider
    print!("├");
    for (i, header) in headers.iter().enumerate() {
        print!("{}", "─".repeat(widths[header] + 2));
        if i < headers.len() - 1 {
            print!("┼");
        }
    }
    println!("┤");

    // Data rows
    for record in records {
        print!("│");
        for (i, header) in headers.iter().enumerate() {
            let val_str = record
                .cells
                .get(header)
                .map(|c| c.value.to_string())
                .unwrap_or_default();
            print!(" {:<width$} ", val_str, width = widths[header]);
            print!("│");
        }
        println!();
    }

    // Bottom border
    print!("╰");
    for (i, header) in headers.iter().enumerate() {
        print!("{}", "─".repeat(widths[header] + 2));
        if i < headers.len() - 1 {
            print!("┴");
        }
    }
    println!("╯");
}

async fn test_misc() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let res1 = approved("valid_name");
    let res2 = approved("");
    let res3 = approved("too_long_name_that_exceeds_thirty_characters");
    let res4 = approved("invalid!char");
    let duration = start.elapsed();

    assert!(res1.is_ok());
    assert!(res2.is_err());
    assert!(res3.is_err());
    assert!(res4.is_err());

    print_bench_table(
        "Misc: approved()",
        vec![
            ("valid_name".to_string(), format!("{:?}", res1)),
            ("empty".to_string(), format!("{:?}", res2)),
            ("too_long".to_string(), format!("{:?}", res3)),
            ("invalid_char".to_string(), format!("{:?}", res4)),
        ],
        duration,
    );
    Ok(())
}

async fn run_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 100;

    // Crypter Benchmarks
    bench_async!("encrypt_token", iterations, encrypt_token("some-token"));
    let encrypted = encrypt_token("some-token").await?;
    bench_async!(
        "decrypt_token",
        iterations,
        decrypt_token(encrypted.clone())
    );

    // Approved Benchmarks
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = approved("some_name");
    }
    let duration = start.elapsed();
    println!(
        "Benchmark {:<30} | Iterations: {:<6} | Total: {:<12?} | Avg: {:?}",
        "approved",
        iterations,
        duration,
        duration / iterations
    );

    Ok(())
}

async fn test_api_tokens(user_service: &UserService) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    // 1. Generate the token using the service
    let raw_token = user_service.create_api_token().await?;
    let duration = start.elapsed();

    // 2. Verify the token exists in the DB.
    // We must hash the raw_token in the query to match the stored hash
    let mut res = DB
        .query("SELECT VALUE user FROM api_token WHERE `token` = crypto::sha512($tokenn) AND is_deleted = false")
        .bind(("tokenn", raw_token.clone()))
        .await?;

    let owner_id: Option<UserId> = res.take(0)?;

    // 3. Assertions
    assert!(owner_id.is_some(), "Token was not found in the database");
    assert_eq!(
        owner_id.unwrap().0,
        user_service.user_record_id.0,
        "Token owner does not match current user"
    );

    print_bench_table(
        "UserService: API Tokens",
        vec![
            ("Generated Token".to_string(), raw_token),
            (
                "DB Verification".to_string(),
                "Passed (SHA-512 Match)".to_string(),
            ),
            ("Owner Match".to_string(), "Verified".to_string()),
        ],
        duration,
    );

    Ok(())
}
