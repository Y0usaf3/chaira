#[macro_export]
macro_rules! env_required {
    ($key:expr) => {{
        use std::env;
        let _ = $crate::dotenvy::dotenv();

        match env::var($key) {
            Ok(val) if !val.trim().is_empty() => val,
            _ => panic!(concat!("Missing required env variable: ", $key)),
        }
    }};
}

use std::sync::LazyLock;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::opt::Config;

pub mod error;
pub use error::Irror;

// TODO: REDIS CON premade connection i suppose

// use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::opt::capabilities::Capabilities;

/* pub static DB: LazyLock<Surreal<Client>> = LazyLock::new(Surreal::init); */
pub static DB: LazyLock<Surreal<Db>> = LazyLock::new(Surreal::init);

pub static CACHE: LazyLock<redis::Client> =
    LazyLock::new(|| redis::Client::open(env_required!("REDIS_URL")).unwrap());

// TODO: use env vars to choose which surli file to use

pub async fn init() {
    let config = Config::default()
        .capabilities(Capabilities::all().with_all_experimental_features_allowed());
    let _ = DB.connect::<Mem>(("memory", config)).await;
    /*  DB.connect::<Ws>(env_required!("DB_URL")).await.unwrap(); */
    // DB.signin(Root {
    //     username: env_required!("DB_USERNAME"),
    //     password: env_required!("DB_PASSWORD"),
    // })
    // .await
    // .unwrap();
    //
    // let bit_path = env_required!("BIT_PATH");
    // let bit_bucket_path = format!("file:/{}", bit_path);

    DB.use_ns("main").use_db("main").await.unwrap();
    // dbg!(
    //     DB.query(
    //         r#"DEFINE BUCKET OVERWRITE bucki BACKEND $bit_bucket_path;
    //            DEFINE MODULE OVERWRITE mod::bit AS f'bucki:/chaira-charli-0.0.1.surli';"#
    //     )
    //     .bind(("bit_bucket_path", bit_bucket_path))
    //     .await
    //     .unwrap()
    // );
    // its a pain to make one search for answers, only to realise the problem was not from it, but
    // from them
    DB.query(include_str!("../../../surql/main.surql"))
        .await
        .unwrap();
}
