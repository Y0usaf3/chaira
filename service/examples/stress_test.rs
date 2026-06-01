use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use models::kinds::*;
use models::*;
use rand::{RngExt, SeedableRng};
use service::base::BaseService;
use service::db::{init, DB};
use service::session::SessionService;
use service::table::{PaginationParams, TableService};
use service::user::{AuthMethod, Session, UserService};
use tokio::sync::Semaphore;

const CONCURRENCY: usize = 40;
const BURST_SECONDS: u64 = 10;
const SEED_RECORDS: usize = 100;
const THINK_MS_MIN: u64 = 50;
const THINK_MS_MAX: u64 = 200;

struct OpStat {
    total_ns: AtomicU64,
    count: AtomicU64,
}

impl OpStat {
    const fn new() -> Self {
        Self { total_ns: AtomicU64::new(0), count: AtomicU64::new(0) }
    }
    fn record(&self, d: Duration) {
        self.total_ns.fetch_add(d.as_nanos() as u64, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }
    fn report(&self, label: &str, w: usize) -> String {
        let c = self.count.load(Ordering::Relaxed);
        let total_ns = self.total_ns.load(Ordering::Relaxed);
        let avg_ns = if c == 0 { 0 } else { total_ns / c };
        format!("  {:<w$} {:>8} {:>10.3}ms  {:>10.3}ms\n",
            label, c, total_ns as f64 / 1_000_000.0, avg_ns as f64 / 1_000_000.0, w = w)
    }
}

struct Stats {
    create_user: OpStat,
    create_session: OpStat,
    login: OpStat,
    create_base: OpStat,
    create_table: OpStat,
    create_field: OpStat,
    create_record: OpStat,
    update_record: OpStat,
    list_records: OpStat,
    get_record: OpStat,
    get_full_data: OpStat,
}

static STATS: Stats = Stats {
    create_user: OpStat::new(),
    create_session: OpStat::new(),
    login: OpStat::new(),
    create_base: OpStat::new(),
    create_table: OpStat::new(),
    create_field: OpStat::new(),
    create_record: OpStat::new(),
    update_record: OpStat::new(),
    list_records: OpStat::new(),
    get_record: OpStat::new(),
    get_full_data: OpStat::new(),
};

static ERRORS: AtomicU64 = AtomicU64::new(0);
static CONCURRENT_PEAK: AtomicU64 = AtomicU64::new(0);

macro_rules! timed {
    ($stat:expr, $expr:expr) => {{
        let __start = Instant::now();
        let __result = $expr;
        $stat.record(__start.elapsed());
        __result
    }};
}

fn field_configs() -> Vec<(&'static str, FieldConfig)> {
    vec![
        ("Name", FieldConfig::Text(TextConfig::SingleLine { default: None, max_length: 200 })),
        ("Age", FieldConfig::Number(NumberConfig::Number { default: None })),
        ("Score", FieldConfig::Number(NumberConfig::Decimal { default: None, precision: 2 })),
        ("Email", FieldConfig::Text(TextConfig::Email)),
    ]
}

fn make_seed_record(id: u64, table_id: &TableId) -> Result<InsertRecord, String> {
    let mut cells = HashMap::new();
    cells.insert("Name".into(), CellValue::new(Value::SingleLine(
        SingleLineValue::new(None, Some(format!("Seed_{}", id))).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Age".into(), CellValue::new(Value::Number(
        NumberValue::new(Some((20 + (id % 40)) as usize), None).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Score".into(), CellValue::new(Value::Decimal(
        DecimalValue::new(Some(id as f64 * 1.5), None).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Email".into(), CellValue::new(Value::Email(
        Email::new(format!("seed{}@test.com", id)).map_err(|e| format!("{e}"))?,
    )));
    Ok(InsertRecord::new(table_id.clone(), cells))
}

fn make_random_record(table_id: &TableId, rng: &mut impl RngExt) -> Result<InsertRecord, String> {
    let mut cells = HashMap::new();
    let id: u64 = rng.random();
    cells.insert("Name".into(), CellValue::new(Value::SingleLine(
        SingleLineValue::new(None, Some(format!("Item_{}", id))).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Age".into(), CellValue::new(Value::Number(
        NumberValue::new(Some((id % 100) as usize), None).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Score".into(), CellValue::new(Value::Decimal(
        DecimalValue::new(Some(rng.random::<f64>() * 100.0), None).map_err(|e| format!("{e}"))?,
    )));
    cells.insert("Email".into(), CellValue::new(Value::Email(
        Email::new(format!("item{}@test.com", id)).map_err(|e| format!("{e}"))?,
    )));
    Ok(InsertRecord::new(table_id.clone(), cells))
}

fn make_random_patch(rng: &mut impl RngExt) -> RecordPatch {
    RecordPatch::new(Some(vec![
        ("Score".into(), CellValue::new(Value::Decimal(
            DecimalValue::new(Some(rng.random::<f64>() * 100.0), None).unwrap(),
        ))),
    ]))
}

async fn burst_worker(worker_id: usize) -> Result<(), String> {
    CONCURRENT_PEAK.fetch_add(1, Ordering::Relaxed);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(worker_id as u64);

    let user: User = timed!(STATS.create_user, DB
        .query("CREATE user SET first_name = $f, last_name = $l, email = $e, role = 'user' RETURN AFTER")
        .bind(("f", format!("W{}", worker_id)))
        .bind(("l", format!("S{}", worker_id)))
        .bind(("e", format!("w{}@test.com", worker_id)))
        .await
        .map_err(|e| format!("create_user: {e}"))?
        .take::<Option<User>>(0)
        .map_err(|e| format!("take user: {e}"))?
        .ok_or("no user returned".to_string())?);

    let user_id = UserId(user.id.as_ref().ok_or("no user id")?.0.clone());

    let (token, _) = timed!(STATS.create_session, SessionService::create_session(
        models::InsertSession { ip: "10.0.0.1".into(), user_agent: "BurstAgent".into(), user: user_id.clone() },
        user,
        true,
    ).await.map_err(|e| format!("create_session: {e}"))?);

    let _us = timed!(STATS.login, UserService::login(AuthMethod::Session(Session {
        token, ip: "10.0.0.1".into(), agent: "BurstAgent".into(),
    })).await.map_err(|e| format!("login: {e}"))?);

    let base = timed!(STATS.create_base, _us.create_base(format!("BurstBase_{}", worker_id)).await.map_err(|e| format!("create_base: {e}"))?);
    let base_id = BaseId(base.id.as_ref().ok_or("no base id")?.0.clone());
    drop(_us);

    let mut bs = BaseService::new(base_id.clone(), user_id.clone()).await.map_err(|e| format!("BaseService::new: {e}"))?;
    let table = timed!(STATS.create_table, bs.create_table("T".into()).await.map_err(|e| format!("create_table: {e}"))?);
    let table_id = TableId(table.id.as_ref().ok_or("no table id")?.0.clone());
    drop(bs);

    {
        let mut ts = TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await.map_err(|e| format!("TableService::new: {e}"))?;
        for (fname, fconfig) in field_configs() {
            timed!(STATS.create_field, ts.create_field(InsertField::new(
                fname.to_string(), fconfig, fname == "Name", true, fname == "Email",
            )).await.map_err(|e| format!("create_field({fname}): {e}"))?);
        }

        let mut seed_records = Vec::with_capacity(SEED_RECORDS);
        for i in 0..SEED_RECORDS as u64 {
            seed_records.push(make_seed_record(i, &table_id)?);
        }
        timed!(STATS.create_record, ts.create_a_lot_of_records(seed_records).await.map_err(|e| format!("seed insert: {e}"))?);
    }

    let end = Instant::now() + Duration::from_secs(BURST_SECONDS);

    loop {
        if Instant::now() >= end { break; }

        let offset: u32 = rng.random_range(0..(SEED_RECORDS - 10).max(1) as u32);

        let mut ts = match timed!(STATS.get_record, TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await) {
            Ok(ts) => ts,
            Err(_) => { ERRORS.fetch_add(1, Ordering::Relaxed); continue; }
        };

        let records = match timed!(STATS.list_records, ts.list_records(PaginationParams { offset: Some(offset), limit: Some(10) }).await) {
            Ok(r) => r,
            Err(_) => { ERRORS.fetch_add(1, Ordering::Relaxed); continue; }
        };

        if let Some(record) = records.first() {
            if rng.random_range(0..2) == 0 {
                let rid = RecordId(record.id.as_ref().unwrap().0.clone());
                let patch = make_random_patch(&mut rng);
                if timed!(STATS.update_record, ts.update_record(rid, patch).await).is_err() {
                    ERRORS.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        drop(ts);

        if Instant::now() >= end { break; }

        if rng.random_range(0..5) == 0 {
            let record = make_random_record(&table_id, &mut rng).map_err(|e| format!("make_record: {e}"))?;
            let mut ts = match TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await {
                Ok(ts) => ts,
                Err(_) => { ERRORS.fetch_add(1, Ordering::Relaxed); continue; }
            };
            if timed!(STATS.create_record, ts.create_record(record).await).is_err() {
                ERRORS.fetch_add(1, Ordering::Relaxed);
            }
        }

        let pause = rng.random_range(THINK_MS_MIN..=THINK_MS_MAX);
        tokio::time::sleep(Duration::from_millis(pause)).await;
    }

    {
        let mut ts = match TableService::new(table_id.clone(), base_id.clone(), user_id.clone()).await {
            Ok(ts) => ts,
            Err(_) => { ERRORS.fetch_add(1, Ordering::Relaxed); return Ok(()); }
        };
        let _ = timed!(STATS.get_full_data, ts.get_full_data(Some(10)).await);
    }

    CONCURRENT_PEAK.fetch_sub(1, Ordering::Relaxed);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init().await;

    println!("═══════════════════════════════════════════════════");
    println!("  STRESS TEST — Per-Function Benchmark");
    println!("═══════════════════════════════════════════════════");
    println!("  Workers:        {} concurrent", CONCURRENCY);
    println!("  Burst:          {}s mixed R/W per worker", BURST_SECONDS);
    println!("  Seed records:   {} per table", SEED_RECORDS);
    println!("  Fields/table:   {}", field_configs().len());
    println!("═══════════════════════════════════════════════════\n");

    let concurrency = Arc::new(Semaphore::new(CONCURRENCY));
    let mut handles = Vec::new();

    println!("Spawning {} workers...", CONCURRENCY);
    let start = Instant::now();

    for i in 0..CONCURRENCY {
        let sem = concurrency.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            if let Err(e) = burst_worker(i).await {
                eprintln!("  ✗ Worker {i} failed: {e}");
                ERRORS.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        h.await.ok();
    }

    let wall = start.elapsed();
    let errs = ERRORS.load(Ordering::Relaxed);
    let peak = CONCURRENT_PEAK.load(Ordering::Relaxed);

    let w = 20;
    println!("\n═══════════════════════════════════════════════════");
    println!("  PER-FUNCTION TIMING");
    println!("═══════════════════════════════════════════════════");
    print!("{}", STATS.create_user.report("create_user", w));
    print!("{}", STATS.create_session.report("create_session", w));
    print!("{}", STATS.login.report("login", w));
    print!("{}", STATS.create_base.report("create_base", w));
    print!("{}", STATS.create_table.report("create_table", w));
    print!("{}", STATS.create_field.report("create_field", w));
    print!("{}", STATS.create_record.report("create_record", w));
    print!("{}", STATS.update_record.report("update_record", w));
    print!("{}", STATS.list_records.report("list_records", w));
    print!("{}", STATS.get_record.report("open_table", w));
    print!("{}", STATS.get_full_data.report("get_full_data", w));

    let total_ops = STATS.create_user.count.load(Ordering::Relaxed)
        + STATS.create_session.count.load(Ordering::Relaxed)
        + STATS.login.count.load(Ordering::Relaxed)
        + STATS.create_base.count.load(Ordering::Relaxed)
        + STATS.create_table.count.load(Ordering::Relaxed)
        + STATS.create_field.count.load(Ordering::Relaxed)
        + STATS.create_record.count.load(Ordering::Relaxed)
        + STATS.update_record.count.load(Ordering::Relaxed)
        + STATS.list_records.count.load(Ordering::Relaxed)
        + STATS.get_record.count.load(Ordering::Relaxed)
        + STATS.get_full_data.count.load(Ordering::Relaxed);

    println!("{}", "─".repeat(60));
    println!("  Total ops:      {total_ops}");
    println!("  Errors:         {errs}");
    println!("  Wall time:      {wall:.3?}");
    println!("  Peak workers:   {peak}");
    println!("  Throughput:     {:.0} ops/sec", total_ops as f64 / wall.as_secs_f64());
    println!("═══════════════════════════════════════════════════\n");

    Ok(())
}
