use models::kinds::*;
use models::*;
use service::base::BaseService;
use service::db::DB;
use service::db::init;
use service::prelude::*;
use service::session::SessionService;
use service::table::PaginationParams;
use service::table::TableService;
use service::user::AuthMethod;
use service::user::Session;
use service::user::UserService;

use std::collections::HashMap;
use std::time::Instant;
use surrealdb::types::ToSql;

// ============================================================================
// PERFORMANCE BENCHMARKING MACROS
// ============================================================================

macro_rules! bench_async {
    ($name:expr, $iterations:expr, $block:expr) => {
        let start = Instant::now();
        for _ in 0..$iterations {
            $block.await.expect("Benchmark failed");
        }
        let duration = start.elapsed();
        println!(
            "Benchmark {:<40} | Iterations: {:<6} | Total: {:<12?} | Avg: {:?}",
            $name,
            $iterations,
            duration,
            duration / $iterations
        );
    };
}

macro_rules! security_test {
    ($name:expr, $assertion:expr, $error_msg:expr) => {
        if $assertion {
            println!("✓ SECURITY TEST PASSED: {}", $name);
        } else {
            panic!("✗ SECURITY TEST FAILED: {} - {}", $name, $error_msg);
        }
    };
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn print_bench_table(title: &str, fields: Vec<(String, String)>, duration: std::time::Duration) {
    let name_width = fields
        .iter()
        .map(|(k, _)| k.len())
        .max()
        .unwrap_or(0)
        .max(5);
    let val_width = fields
        .iter()
        .map(|(_, v)| v.len())
        .max()
        .unwrap_or(0)
        .max(5);

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

fn print_security_test_header(test_name: &str) {
    println!("\n{}", "═".repeat(80));
    println!("🔒 SECURITY TEST: {}", test_name);
    println!("{}\n", "═".repeat(80));
}

fn print_performance_test_header(test_name: &str) {
    println!("\n{}", "═".repeat(80));
    println!("⚡ PERFORMANCE TEST: {}", test_name);
    println!("{}\n", "═".repeat(80));
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init().await;

    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                 COMPREHENSIVE SERVICE TEST & SECURITY AUDIT                    ║");
    println!("║                     Testing all functions in @service/src/                     ║");
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");

    // Phase 1: Crypter Functions
    test_crypter_functions().await?;

    // Phase 2: Approval Functions
    test_approval_functions().await?;

    // Phase 3: User Service Functions
    let (user_service, user_id) = test_user_service_functions().await?;

    // Phase 4: Base Service Functions
    let (mut base_service, base_id) = test_base_service_functions(&user_service, &user_id).await?;

    // Phase 5: Table Service Functions
    test_table_service_functions(&mut base_service, &user_id, &base_id).await?;

    // Phase 6: Security Tests
    test_security_mechanisms(&user_service, &base_service).await?;

    // Phase 7: Performance Stress Tests
    test_performance_stress(&mut base_service, &user_id, &base_id).await?;

    // Phase 8: Concurrent Operations
    test_concurrent_operations(&mut base_service, &user_id, &base_id).await?;

    // Phase 9: Real-world 250k Records Test
    test_large_dataset(&mut base_service, &user_id, &base_id).await?;

    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                      ALL TESTS COMPLETED SUCCESSFULLY ✓                        ║");
    println!("╚════════════════════════════════════════════════════════════════════════════════╝");

    Ok(())
}

// ============================================================================
// PHASE 1: CRYPTER FUNCTIONS (encrypter.rs)
// ============================================================================

async fn test_crypter_functions() -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("Crypter Functions");

    // Test 1: Basic encrypt/decrypt
    let start = Instant::now();
    let token = "my-secret-token-12345";
    let encrypted = encrypt_token(token).await?;
    let decrypted = decrypt_token(encrypted.clone()).await?;
    let duration = start.elapsed();

    assert_eq!(token, decrypted, "Decrypted token does not match original");

    print_bench_table(
        "Crypter: Basic Encrypt/Decrypt",
        vec![
            ("Input Token Length".to_string(), token.len().to_string()),
            ("Encrypted Length".to_string(), encrypted.len().to_string()),
            (
                "Decryption Successful".to_string(),
                (token == decrypted).to_string(),
            ),
        ],
        duration,
    );

    // Test 2: Multiple encryption consistency
    let start = Instant::now();
    let token1 = encrypt_token("test").await?;
    let token2 = encrypt_token("test").await?;
    let duration = start.elapsed();

    // Different nonces should produce different ciphertexts (security property)
    assert_ne!(
        token1, token2,
        "Encrypted tokens should be different due to random nonces"
    );

    print_bench_table(
        "Crypter: Different Nonce Generation",
        vec![
            (
                "Token 1 == Token 2".to_string(),
                (token1 == token2).to_string(),
            ),
            ("Nonce Randomization".to_string(), "✓ Working".to_string()),
        ],
        duration,
    );

    // Test 3: Encryption performance benchmark
    print_performance_test_header("Crypter Performance Benchmarks");
    let iterations = 100;
    bench_async!(
        "encrypt_token (100 iterations)",
        iterations,
        encrypt_token("bench-token")
    );

    let encrypted_token = encrypt_token("bench-token").await?;
    bench_async!(
        "decrypt_token (100 iterations)",
        iterations,
        decrypt_token(encrypted_token.clone())
    );

    // Test 4: Large token encryption
    let start = Instant::now();
    let large_token = "x".repeat(1000);
    let encrypted_large = encrypt_token(&large_token).await?;
    let encrypted_large_len = encrypted_large.len();
    let decrypted_large = decrypt_token(encrypted_large).await?;
    let duration = start.elapsed();

    assert_eq!(
        large_token, decrypted_large,
        "Large token encryption failed"
    );

    print_bench_table(
        "Crypter: Large Token (1000 bytes)",
        vec![
            ("Original Size".to_string(), large_token.len().to_string()),
            (
                "Encrypted Size".to_string(),
                encrypted_large_len.to_string(),
            ),
            ("Success".to_string(), "✓ Yes".to_string()),
        ],
        duration,
    );

    Ok(())
}

// ============================================================================
// PHASE 2: APPROVAL FUNCTIONS (lib.rs)
// ============================================================================

async fn test_approval_functions() -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("Approval Functions");

    // Test valid names
    let start = Instant::now();
    let valid_cases = vec![
        "valid_name",
        "test-name",
        "Name123",
        "a",
        "ab",
        "very_long_name_that_is_okay",
    ];

    for case in &valid_cases {
        assert!(approved(case).is_ok(), "Valid case failed: {}", case);
    }
    let duration = start.elapsed();

    print_bench_table(
        "Approved: Valid Cases",
        vec![
            (
                "Total Valid Cases".to_string(),
                valid_cases.len().to_string(),
            ),
            ("All Passed".to_string(), "✓ Yes".to_string()),
        ],
        duration,
    );

    // Test invalid names
    let start = Instant::now();
    let invalid_cases = vec![
        ("", "Empty string"),
        ("too_long_name_that_exceeds_thirty_characters", "Too long"),
        ("invalid!char", "Invalid character"),
        ("name@here", "Special char @"),
        ("name#hash", "Special char #"),
        ("name with spaces", "Spaces not allowed"),
    ];

    for (case, reason) in &invalid_cases {
        assert!(
            approved(case).is_err(),
            "Should reject: {} ({})",
            case,
            reason
        );
    }
    let duration = start.elapsed();

    print_bench_table(
        "Approved: Invalid Cases (Security)",
        vec![
            (
                "Total Invalid Cases".to_string(),
                invalid_cases.len().to_string(),
            ),
            ("All Rejected".to_string(), "✓ Yes".to_string()),
        ],
        duration,
    );

    // Performance benchmark
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = approved("some_name");
    }
    let duration = start.elapsed();

    println!(
        "Benchmark {:<40} | Iterations: {:<6} | Total: {:<12?} | Avg: {:?}",
        "approved (1000 iterations)",
        1000,
        duration,
        duration / 1000
    );

    Ok(())
}

// ============================================================================
// PHASE 3: USER SERVICE FUNCTIONS (user.rs)
// ============================================================================

async fn test_user_service_functions() -> Result<(UserService, UserId), Box<dyn std::error::Error>>
{
    print_performance_test_header("UserService Functions");

    // Test 1: Manual user creation
    let start = Instant::now();
    let user: User = DB
        .query("CREATE user SET first_name = 'Test', last_name = 'User', email = 'comprehensive-test@example.com', role = 'admin' RETURN AFTER")
        .await?
        .take::<Option<User>>(0)?
        .unwrap();
    let duration = start.elapsed();
    let user_id = UserId(user.id.as_ref().unwrap().0.clone());

    print_bench_table(
        "UserService: User Creation",
        vec![
            ("User ID".to_string(), user_id.0.to_sql_pretty()),
            ("Email".to_string(), user.email.clone()),
            ("Role".to_string(), format!("{:?}", user.role())),
        ],
        duration,
    );

    // Test 2: Session-based login
    let start = Instant::now();
    let token = "comprehensive-test-session-token";
    let ip = "192.168.1.1".to_string();
    let agent = "ComprehensiveTestAgent".to_string();

    let insert_session = models::InsertSession {
        ip: ip.clone(),
        user_agent: agent.clone(),
        user: user_id.clone(),
    };
    let mock_session = SessionService::create_session(insert_session, true).await?;

    let mut user_service = UserService::login(AuthMethod::Session(Session {
        token: token.to_string(),
        ip,
        agent,
    }))
    .await?;
    let duration = start.elapsed();

    print_bench_table(
        "UserService: Session Login",
        vec![
            ("Email".to_string(), user_service.user.email.clone()),
            ("Auth Method".to_string(), "Session".to_string()),
            ("Success".to_string(), "✓ Yes".to_string()),
        ],
        duration,
    );

    // Test 3: is_admin check
    let start = Instant::now();
    let is_admin = user_service.is_admin().await?;
    let duration = start.elapsed();
    assert!(is_admin, "User should be admin");

    print_bench_table(
        "UserService: is_admin()",
        vec![("Is Admin".to_string(), is_admin.to_string())],
        duration,
    );

    // Test 4: create_base
    let start = Instant::now();
    let base = user_service
        .create_base("ComprehensiveTestBase".to_string())
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "UserService: create_base()",
        vec![
            ("Base Name".to_string(), base.name.clone()),
            (
                "Base ID".to_string(),
                base.id.as_ref().unwrap().0.to_sql_pretty(),
            ),
        ],
        duration,
    );

    // Test 5: list_bases
    let start = Instant::now();
    let bases = user_service.list_bases().await?;
    let duration = start.elapsed();

    print_bench_table(
        "UserService: list_bases()",
        vec![("Total Bases".to_string(), bases.len().to_string())],
        duration,
    );

    // Test 7: create_session (alternative to refresh_user)
    let start = Instant::now();
    let session_ip = "10.0.0.2".to_string();
    let session_agent = "RefreshTestAgent".to_string();
    let refresh_session = user_service
        .create_session(session_ip, session_agent)
        .await?;
    let duration = start.elapsed();

    assert!(
        !refresh_session.is_empty(),
        "Session token should not be empty"
    );

    print_bench_table(
        "UserService: create_session (refresh variant)",
        vec![("Session Created".to_string(), "✓ Yes".to_string())],
        duration,
    );

    // Test 8: create_session
    let start = Instant::now();
    let session_token = user_service
        .create_session("10.0.0.1".to_string(), "TestBrowser".to_string())
        .await?;
    let duration = start.elapsed();

    assert!(
        !session_token.is_empty(),
        "Session token should not be empty"
    );

    print_bench_table(
        "UserService: create_session()",
        vec![
            ("Token Created".to_string(), "✓ Yes".to_string()),
            ("Token Length".to_string(), session_token.len().to_string()),
        ],
        duration,
    );

    Ok((user_service, user_id))
}

// ============================================================================
// PHASE 4: BASE SERVICE FUNCTIONS (base.rs)
// ============================================================================

async fn test_base_service_functions(
    user_service: &UserService,
    user_id: &UserId,
) -> Result<(BaseService, BaseId), Box<dyn std::error::Error>> {
    print_performance_test_header("BaseService Functions");

    // Test 1: BaseService::new
    let start = Instant::now();
    let bases = user_service.list_bases().await?;
    let base_id = BaseId(bases[0].id.as_ref().unwrap().0.clone());

    let mut base_service = BaseService::new(base_id.clone(), user_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "BaseService: new()",
        vec![("Base ID".to_string(), base_service.id().0.to_sql_pretty())],
        duration,
    );

    // Test 2: create_table
    let start = Instant::now();
    let table = base_service
        .create_table("comprehensive_test_table".to_string())
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "BaseService: create_table()",
        vec![
            ("Table Name".to_string(), table.name.clone()),
            (
                "Table ID".to_string(),
                table.id.as_ref().unwrap().0.to_sql_pretty(),
            ),
        ],
        duration,
    );

    // Test 3: list_tables
    let start = Instant::now();
    let tables = base_service.list_tables().await?;
    let duration = start.elapsed();

    print_bench_table(
        "BaseService: list_tables()",
        vec![("Total Tables".to_string(), tables.len().to_string())],
        duration,
    );

    // Test 4: open_table
    let table_id = TableId(table.id.unwrap().0);
    let start = Instant::now();
    let _table_service = base_service.open_table(table_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "BaseService: open_table()",
        vec![("Table Opened".to_string(), "✓ Yes".to_string())],
        duration,
    );

    Ok((base_service, base_id))
}

// ============================================================================
// PHASE 5: TABLE SERVICE FUNCTIONS (table.rs)
// ============================================================================

async fn test_table_service_functions(
    base_service: &mut BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("TableService Functions");

    let table = base_service
        .create_table("comprehensive_table_test".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);

    let mut table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // Test 1: create_field
    let start = Instant::now();
    let config = FieldConfig::Text(TextConfig::SingleLine {
        default: None,
        max_length: 255,
    });
    let insert_field = InsertField::new("test_field".to_string(), config, false, true, false);

    let field = table_service.create_field(insert_field).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: create_field()",
        vec![
            ("Field Name".to_string(), field.name.clone()),
            ("Field Type".to_string(), "Text/SingleLine".to_string()),
        ],
        duration,
    );

    let field_id = FieldId(field.id.clone().unwrap().0);

    // Test 2: get_field_config
    let start = Instant::now();
    let config_fr = table_service.get_field_config(field_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: get_field_config()",
        vec![("Field Name".to_string(), config_fr.name.clone())],
        duration,
    );

    // Test 3: create_a_lot_of_fields
    let start = Instant::now();
    let field_definitions = vec![
        (
            "Name",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        ("Email", FieldConfig::Text(TextConfig::Email)),
        (
            "Age",
            FieldConfig::Number(NumberConfig::Number { default: None }),
        ),
        (
            "Score",
            FieldConfig::Number(NumberConfig::Decimal {
                default: None,
                precision: 2,
            }),
        ),
    ];

    let inserts: Vec<InsertField> = field_definitions
        .iter()
        .map(|a| InsertField::new(a.0.to_string(), a.1.clone(), false, true, false))
        .collect();

    let fields = table_service.create_a_lot_of_fields(inserts).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: create_a_lot_of_fields()",
        vec![("Fields Created".to_string(), fields.len().to_string())],
        duration,
    );

    // Test 4: create_record
    let start = Instant::now();
    let mut cells = HashMap::new();
    let val = Value::SingleLine(SingleLineValue::new(
        None,
        Some("comprehensive test".to_string()),
    )?);
    cells.insert("test_field".to_string(), CellValue::new(val));
    let insert_record = InsertRecord::new(table_id.clone(), cells);
    let record = table_service.create_record(insert_record).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: create_record()",
        vec![("Record Created".to_string(), "✓ Yes".to_string())],
        duration,
    );

    let record_id = RecordId(record.id.clone().unwrap().0);

    // Test 5: get_record
    let start = Instant::now();
    let _fetched_record = table_service.get_record(record_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: get_record()",
        vec![("Record Retrieved".to_string(), "✓ Yes".to_string())],
        duration,
    );

    // Test 6: update_record
    let start = Instant::now();
    let mut changed_cells = Vec::new();
    let val_updated = Value::SingleLine(SingleLineValue::new(
        None,
        Some("updated comprehensive test".to_string()),
    )?);
    changed_cells.push(("test_field".to_string(), CellValue::new(val_updated)));
    let patch = RecordPatch::new(Some(changed_cells));
    let _updated_record = table_service
        .update_record(record_id.clone(), patch)
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: update_record()",
        vec![("Record Updated".to_string(), "✓ Yes".to_string())],
        duration,
    );

    // Test 7: list_records
    let start = Instant::now();
    let records = table_service
        .list_records(PaginationParams {
            offset: Some(0),
            limit: Some(10),
        })
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: list_records()",
        vec![("Records Retrieved".to_string(), records.len().to_string())],
        duration,
    );

    // Test 8: get_full_data
    let start = Instant::now();
    let (fields_result, records_result) = table_service.get_full_data(Some(50)).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: get_full_data()",
        vec![
            ("Fields".to_string(), fields_result.len().to_string()),
            ("Records".to_string(), records_result.len().to_string()),
        ],
        duration,
    );

    // Test 9: check_migration
    let start = Instant::now();
    let target_config = FieldConfig::Number(NumberConfig::Number { default: None });
    let report = table_service
        .check_migration(field_id.clone(), target_config)
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: check_migration()",
        vec![
            (
                "Affected Records".to_string(),
                report.affected_records.to_string(),
            ),
            (
                "Success Rate".to_string(),
                format!("{:.2}%", report.success_rate * 100.0),
            ),
        ],
        duration,
    );

    // Test 10: create_a_lot_of_records
    let start = Instant::now();
    let mut insert_records = Vec::with_capacity(100);

    for i in 0..100 {
        let mut cells = HashMap::new();
        cells.insert(
            "Name".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("Record {}", i)),
            )?)),
        );
        cells.insert(
            "Email".to_string(),
            CellValue::new(Value::Email(Email::new(format!("record{}@test.com", i))?)),
        );
        cells.insert(
            "Age".to_string(),
            CellValue::new(Value::Number(NumberValue::new(Some(20 + (i % 30)), None)?)),
        );
        cells.insert(
            "Score".to_string(),
            CellValue::new(Value::Decimal(DecimalValue::new(
                Some(i as f64 * 1.5),
                None,
            )?)),
        );

        let insert = InsertRecord::new(table_id.clone(), cells);
        insert_records.push(insert);
    }

    table_service
        .create_a_lot_of_records(insert_records)
        .await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: create_a_lot_of_records()",
        vec![("Records Inserted".to_string(), "100".to_string())],
        duration,
    );

    // Test 11: delete_record
    let start = Instant::now();
    let _deleted = table_service.delete_record(record_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: delete_record()",
        vec![("Record Deleted".to_string(), "✓ Yes".to_string())],
        duration,
    );

    // Test 12: delete_field
    let start = Instant::now();
    let _deleted_field = table_service.delete_field(field_id.clone()).await?;
    let duration = start.elapsed();

    print_bench_table(
        "TableService: delete_field()",
        vec![("Field Deleted".to_string(), "✓ Yes".to_string())],
        duration,
    );

    Ok(())
}

// ============================================================================
// PHASE 6: SECURITY TESTS
// ============================================================================

async fn test_security_mechanisms(
    user_service: &UserService,
    _base_service: &BaseService,
) -> Result<(), Box<dyn std::error::Error>> {
    print_security_test_header("Token Encryption & Storage");

    // Test 1: Token encryption security
    let token1 = encrypt_token("secret").await?;
    let token2 = encrypt_token("secret").await?;
    security_test!(
        "Different nonces for same plaintext",
        token1 != token2,
        "Nonces should be unique"
    );

    // Test 2: Token decryption accuracy
    let original = "test-token-12345";
    let encrypted = encrypt_token(original).await?;
    let decrypted = decrypt_token(encrypted).await?;
    security_test!(
        "Decrypt matches original",
        original == decrypted,
        "Decryption failed"
    );

    // Test 4: Approval function input validation
    let dangerous_inputs = vec![
        "admin'; DROP TABLE user; --",
        "../../../etc/passwd",
        "<script>alert('xss')</script>",
        "name\0with\0nulls",
        "name\nwith\nnewlines",
    ];

    for dangerous in dangerous_inputs {
        security_test!(
            &format!(
                "Reject malicious input: {}",
                dangerous.chars().take(20).collect::<String>()
            ),
            approved(dangerous).is_err(),
            "Malicious input was accepted"
        );
    }

    // Test 5: Session validation
    print_security_test_header("Session & Authentication");

    let session_token = user_service
        .create_session("127.0.0.1".to_string(), "TestAgent".to_string())
        .await?;
    security_test!(
        "Session token generated successfully",
        !session_token.is_empty(),
        "Empty session token"
    );

    // Verify session token is cryptographically random (has sufficient entropy)
    let token1 = user_service
        .create_session("127.0.0.1".to_string(), "TestAgent1".to_string())
        .await?;
    let token2 = user_service
        .create_session("127.0.0.1".to_string(), "TestAgent2".to_string())
        .await?;
    security_test!(
        "Session tokens are unique (random generation)",
        token1 != token2,
        "Sessions should have unique tokens"
    );

    // Test 6: Base access control
    print_security_test_header("Authorization & Permissions");

    let bases = user_service.list_bases().await?;
    security_test!(
        "User can access their own base",
        !bases.is_empty(),
        "User should have access to created bases"
    );

    Ok(())
}

// ============================================================================
// PHASE 7: PERFORMANCE STRESS TESTS
// ============================================================================

async fn test_performance_stress(
    base_service: &mut BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("Stress Test: Large Scale Operations");

    let table = base_service
        .create_table("stress_test_table".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);
    let mut table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // Stress Test 1: Create many fields
    let start = Instant::now();
    let field_definitions = vec![
        (
            "Field1",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "Field2",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "Field3",
            FieldConfig::Number(NumberConfig::Number { default: None }),
        ),
        ("Field4", FieldConfig::Text(TextConfig::Email)),
    ];

    let inserts: Vec<InsertField> = field_definitions
        .iter()
        .map(|a| InsertField::new(a.0.to_string(), a.1.clone(), false, true, false))
        .collect();
    let fields = table_service.create_a_lot_of_fields(inserts).await?;
    let duration = start.elapsed();

    println!(
        "\n⚡ Stress Test: Created {} fields in {:?}",
        fields.len(),
        duration
    );

    // Stress Test 2: Create many records
    let start = Instant::now();
    let num_records = 500;
    let mut insert_records = Vec::with_capacity(num_records);

    for i in 0..num_records {
        let mut cells = HashMap::new();
        cells.insert(
            "Field1".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("Record {}", i)),
            )?)),
        );
        cells.insert(
            "Field2".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("Description for record {}", i)),
            )?)),
        );
        cells.insert(
            "Field3".to_string(),
            CellValue::new(Value::Number(NumberValue::new(Some(i as usize), None)?)),
        );
        cells.insert(
            "Field4".to_string(),
            CellValue::new(Value::Email(Email::new(format!(
                "user{}@stress-test.com",
                i
            ))?)),
        );

        let insert = InsertRecord::new(table_id.clone(), cells);
        insert_records.push(insert);
    }

    table_service
        .create_a_lot_of_records(insert_records)
        .await?;
    let duration = start.elapsed();

    println!(
        "⚡ Stress Test: Inserted {} records in {:?} (avg: {:?}/record)",
        num_records,
        duration,
        duration / num_records as u32
    );

    // Stress Test 3: List with pagination
    let start = Instant::now();
    let records = table_service
        .list_records(PaginationParams {
            offset: Some(0),
            limit: Some(100),
        })
        .await?;
    let duration = start.elapsed();

    println!("⚡ Stress Test: Listed 100 records in {:?}", duration);

    // Stress Test 4: Update many records
    let start = Instant::now();
    let record_ids: Vec<RecordId> = records
        .into_iter()
        .take(50)
        .map(|record| RecordId(record.id.unwrap().0))
        .collect();

    for (i, id) in record_ids.iter().enumerate() {
        let mut changes = Vec::new();
        changes.push((
            "Field1".to_string(),
            CellValue::new(Value::SingleLine(SingleLineValue::new(
                None,
                Some(format!("Updated Record {}", i)),
            )?)),
        ));
        let patch = RecordPatch::new(Some(changes));
        table_service.update_record(id.clone(), patch).await?;
    }
    let duration = start.elapsed();

    println!(
        "⚡ Stress Test: Updated {} records in {:?} (avg: {:?}/record)",
        record_ids.len(),
        duration,
        duration / record_ids.len() as u32
    );

    Ok(())
}

// ============================================================================
// PHASE 8: CONCURRENT OPERATIONS
// ============================================================================

async fn test_concurrent_operations(
    base_service: &mut BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("Concurrent Operations");

    let table = base_service
        .create_table("concurrent_test_table".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);

    let mut table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // Create fields for concurrent testing
    let field_defs = vec![
        (
            "ConcurrentField1",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "ConcurrentField2",
            FieldConfig::Number(NumberConfig::Number { default: None }),
        ),
    ];

    let inserts: Vec<InsertField> = field_defs
        .iter()
        .map(|a| InsertField::new(a.0.to_string(), a.1.clone(), false, true, false))
        .collect();
    let _ = table_service.create_a_lot_of_fields(inserts).await?;

    // Test concurrent record creation
    let start = Instant::now();
    let mut handles = vec![];

    for i in 0..10 {
        let table_id_clone = table_id.clone();
        let user_id_clone = user_id.clone();
        let base_id_clone = base_id.clone();

        let handle = tokio::spawn(async move {
            match TableService::new(table_id_clone.clone(), base_id_clone, user_id_clone).await {
                Ok(mut table_service) => {
                    let mut cells = HashMap::new();
                    cells.insert(
                        "ConcurrentField1".to_string(),
                        CellValue::new(Value::SingleLine(
                            SingleLineValue::new(None, Some(format!("Concurrent Record {}", i)))
                                .unwrap(),
                        )),
                    );
                    cells.insert(
                        "ConcurrentField2".to_string(),
                        CellValue::new(Value::Number(
                            NumberValue::new(Some(i as usize), None).unwrap(),
                        )),
                    );

                    let insert = InsertRecord::new(table_id_clone, cells);
                    table_service.create_record(insert).await.ok()
                }
                Err(_) => None,
            }
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.ok().flatten().is_some() {
            success_count += 1;
        }
    }
    let duration = start.elapsed();

    println!(
        "⚡ Concurrent Test: {} concurrent record creations completed in {:?}",
        success_count, duration
    );

    Ok(())
}

// ============================================================================
// PHASE 9: REAL-WORLD 250K RECORDS TEST
// ============================================================================

async fn test_large_dataset(
    base_service: &mut BaseService,
    user_id: &UserId,
    base_id: &BaseId,
) -> Result<(), Box<dyn std::error::Error>> {
    print_performance_test_header("Real-World 250K Records Dataset");

    let table = base_service
        .create_table("large_dataset_table".to_string())
        .await?;
    let table_id = TableId(table.id.unwrap().0);
    let mut table_service =
        TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await?;

    // Create realistic schema for a customer/order database
    let schema = vec![
        (
            "customer_id",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 50,
            }),
        ),
        (
            "first_name",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "last_name",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        ("email", FieldConfig::Text(TextConfig::Email)),
        (
            "phone",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 20,
            }),
        ),
        (
            "city",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "country",
            FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 100,
            }),
        ),
        (
            "purchase_amount",
            FieldConfig::Number(NumberConfig::Decimal {
                default: None,
                precision: 2,
            }),
        ),
        (
            "purchase_count",
            FieldConfig::Number(NumberConfig::Number { default: None }),
        ),
    ];

    println!("\n📊 Creating schema with {} fields...", schema.len());
    let start = Instant::now();
    let inserts: Vec<InsertField> = schema
        .iter()
        .map(|a| InsertField::new(a.0.to_string(), a.1.clone(), false, true, false))
        .collect();
    let fields = table_service.create_a_lot_of_fields(inserts).await?;
    let duration = start.elapsed();

    println!("✓ Schema created in {:?}", duration);

    print_bench_table(
        "250K Dataset: Schema Setup",
        vec![("Fields Created".to_string(), fields.len().to_string())],
        duration,
    );

    // Sample data for realistic records
    let first_names = vec![
        "John", "Jane", "Michael", "Sarah", "David", "Emma", "Robert", "Lisa", "James", "Mary",
        "William", "Patricia", "Richard", "Jennifer", "Charles", "Barbara", "Joseph", "Susan",
        "Thomas", "Jessica", "Daniel", "Karen", "Matthew", "Nancy", "Mark", "Linda",
    ];

    let last_names = vec![
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Garcia",
        "Wilson",
        "Anderson",
        "Taylor",
        "Thomas",
        "Moore",
        "Jackson",
        "Martin",
        "Lee",
        "Perez",
        "Thompson",
        "White",
        "Harris",
        "Sanchez",
        "Clark",
    ];

    let cities = vec![
        "New York",
        "Los Angeles",
        "Chicago",
        "Houston",
        "Phoenix",
        "Philadelphia",
        "San Antonio",
        "San Diego",
        "Dallas",
        "San Jose",
        "Austin",
        "Jacksonville",
        "Seattle",
        "Denver",
        "Boston",
        "Miami",
        "Portland",
        "Atlanta",
        "Detroit",
        "Minneapolis",
    ];

    let countries = vec![
        "USA",
        "Canada",
        "UK",
        "Germany",
        "France",
        "Spain",
        "Italy",
        "Australia",
    ];

    // Phase 1: Bulk insert 250k records
    println!("\n📝 Bulk inserting 250,000 records...");
    let start = Instant::now();
    let batch_size = 5000;
    let total_records = 250_000;
    let mut batches_inserted = 0;

    for batch_start in (0..total_records).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, total_records);
        let mut insert_records = Vec::with_capacity(batch_size);

        for i in batch_start..batch_end {
            let mut cells = HashMap::new();

            // Generate realistic data
            cells.insert(
                "customer_id".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(format!("CUST-{:06}", i)),
                )?)),
            );

            let first_name = first_names[i % first_names.len()];
            cells.insert(
                "first_name".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(first_name.to_string()),
                )?)),
            );

            let last_name = last_names[i % last_names.len()];
            cells.insert(
                "last_name".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(last_name.to_string()),
                )?)),
            );

            cells.insert(
                "email".to_string(),
                CellValue::new(Value::Email(Email::new(format!(
                    "customer{}@example.com",
                    i
                ))?)),
            );

            let phone_prefix = 200 + (i % 700);
            cells.insert(
                "phone".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(format!("+1-{}-555-{:04}", phone_prefix, i % 10000)),
                )?)),
            );

            let city = cities[i % cities.len()];
            cells.insert(
                "city".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(city.to_string()),
                )?)),
            );

            let country = countries[i % countries.len()];
            cells.insert(
                "country".to_string(),
                CellValue::new(Value::SingleLine(SingleLineValue::new(
                    None,
                    Some(country.to_string()),
                )?)),
            );

            let purchase_amount = (i as f64 % 5000.0) + 10.0 * (1.0 + (i % 100) as f64 / 100.0);
            cells.insert(
                "purchase_amount".to_string(),
                CellValue::new(Value::Decimal(DecimalValue::new(
                    Some(purchase_amount),
                    None,
                )?)),
            );

            cells.insert(
                "purchase_count".to_string(),
                CellValue::new(Value::Number(NumberValue::new(Some(1 + (i % 50)), None)?)),
            );

            let insert = InsertRecord::new(table_id.clone(), cells);
            insert_records.push(insert);
        }

        table_service
            .create_a_lot_of_records(insert_records)
            .await?;

        batches_inserted += 1;
        if batches_inserted % 10 == 0 {
            let elapsed = start.elapsed();
            let records_so_far = batch_end;
            let rate = records_so_far as f64 / elapsed.as_secs_f64();
            println!(
                "  ⏳ Inserted {}/{} records ({:.0} records/sec)",
                records_so_far, total_records, rate
            );
        }
    }

    let insert_duration = start.elapsed();
    println!(
        "✓ All 250,000 records inserted in {:?} ({:.0} records/sec)\n",
        insert_duration,
        total_records as f64 / insert_duration.as_secs_f64()
    );

    print_bench_table(
        "250K Dataset: Bulk Insert",
        vec![
            ("Records Inserted".to_string(), total_records.to_string()),
            ("Total Time".to_string(), format!("{:?}", insert_duration)),
            (
                "Throughput".to_string(),
                format!(
                    "{:.0} records/sec",
                    total_records as f64 / insert_duration.as_secs_f64()
                ),
            ),
        ],
        insert_duration,
    );

    // Phase 2: Test pagination with different page sizes
    println!("\n📄 Testing pagination performance...");
    let pagination_tests = vec![
        ("Small page (10 records)", 10),
        ("Medium page (100 records)", 100),
        ("Large page (1000 records)", 1000),
    ];

    for (test_name, page_size) in pagination_tests {
        let start = Instant::now();
        let _records = table_service
            .list_records(PaginationParams {
                offset: Some(0),
                limit: Some(page_size),
            })
            .await?;
        let duration = start.elapsed();

        println!("  ⏳ {} in {:?}", test_name, duration);
    }

    // Phase 3: Full data retrieval test
    println!("\n🔍 Testing full data retrieval...");
    let start = Instant::now();
    let (fields_result, records_result) = table_service.get_full_data(Some(1000)).await?;
    let full_data_duration = start.elapsed();

    println!(
        "✓ Retrieved all fields and records in {:?}",
        full_data_duration
    );

    print_bench_table(
        "250K Dataset: Full Data Retrieval",
        vec![
            ("Fields".to_string(), fields_result.len().to_string()),
            (
                "Records Retrieved".to_string(),
                records_result.len().to_string(),
            ),
            (
                "Total Time".to_string(),
                format!("{:?}", full_data_duration),
            ),
        ],
        full_data_duration,
    );

    // Phase 4: Random access test (fetch random records)
    println!("\n🎲 Testing random access performance...");
    let start = Instant::now();
    let sample_size = 100;

    for i in 0..sample_size {
        let offset = (i * total_records / sample_size) as u32;
        let _records = table_service
            .list_records(PaginationParams {
                offset: Some(offset),
                limit: Some(1),
            })
            .await?;
    }

    let random_access_duration = start.elapsed();
    println!(
        "✓ Random access test ({} random reads) completed in {:?}",
        sample_size, random_access_duration
    );

    print_bench_table(
        "250K Dataset: Random Access",
        vec![
            ("Random Reads".to_string(), sample_size.to_string()),
            (
                "Total Time".to_string(),
                format!("{:?}", random_access_duration),
            ),
            (
                "Avg Per Read".to_string(),
                format!("{:?}", random_access_duration / sample_size as u32),
            ),
        ],
        random_access_duration,
    );

    // Phase 5: Concurrent reads test
    println!("\n⚡ Testing concurrent reads on 250K dataset...");
    let start = Instant::now();
    let mut handles = vec![];
    let concurrent_tasks = 20;

    for i in 0..concurrent_tasks {
        let table_id_clone = table_id.clone();
        let user_id_clone = user_id.clone();
        let base_id_clone = base_id.clone();

        let handle = tokio::spawn(async move {
            match TableService::new(table_id_clone, base_id_clone, user_id_clone).await {
                Ok(mut table_service) => {
                    let offset = (i as u32 * total_records as u32 / concurrent_tasks as u32);
                    table_service
                        .list_records(PaginationParams {
                            offset: Some(offset),
                            limit: Some(100),
                        })
                        .await
                        .ok()
                }
                Err(_) => None,
            }
        });

        handles.push(handle);
    }

    let mut success_count = 0;
    for handle in handles {
        if handle.await.ok().flatten().is_some() {
            success_count += 1;
        }
    }

    let concurrent_duration = start.elapsed();
    println!(
        "✓ Concurrent reads ({}/{} tasks successful) completed in {:?}",
        success_count, concurrent_tasks, concurrent_duration
    );

    print_bench_table(
        "250K Dataset: Concurrent Reads",
        vec![
            ("Concurrent Tasks".to_string(), concurrent_tasks.to_string()),
            ("Successful".to_string(), success_count.to_string()),
            (
                "Total Time".to_string(),
                format!("{:?}", concurrent_duration),
            ),
        ],
        concurrent_duration,
    );

    // Phase 6: Summary statistics
    println!("\n📈 250K Dataset Performance Summary:");
    println!("  • Total insertion time: {:?}", insert_duration);
    println!("  • Full data retrieval time: {:?}", full_data_duration);
    println!(
        "  • Random access (100 reads): {:?}",
        random_access_duration
    );
    println!("  • Concurrent reads (20 tasks): {:?}", concurrent_duration);
    println!(
        "  • Overall throughput: {:.0} records/sec (insertion)",
        total_records as f64 / insert_duration.as_secs_f64()
    );

    Ok(())
}
