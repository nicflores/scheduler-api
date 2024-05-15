use crate::settings::AppConfig;
use async_trait::async_trait;
use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Extension, Json, Router,
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use chrono::{DateTime, Duration, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, str::FromStr, sync::Arc};
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time,
};
//use tower_http::trace::TraceLayer;

pub fn create_scheduler_app<T: SchedulerRepo>(scheduler_repo: T, app_config: AppConfig) -> Router {
    let arc_app_config = Arc::new(app_config);
    Router::new()
        .route("/schedules", get(get_all_schedules::<T>))
        .route("/schedules", post(create_a_schedule::<T>))
        .route("/schedules/:id", get(get_schedule::<T>))
        .route("/schedules/:id", put(update_schedule::<T>))
        .route("/schedules/:id", delete(delete_schedule::<T>))
        .route("/schedules/cron", post(get_next_runs))
        .route("/health", get(health))
        .route("/config", get(config))
        .with_state(scheduler_repo)
        .layer(Extension(arc_app_config))
        .layer(OtelInResponseLayer::default())
        .layer(OtelAxumLayer::default())
    //.layer(TraceLayer::new_for_http())
}

async fn get_next_runs(Json(cron): Json<Cron>) -> Json<NextRuns> {
    let schedule = cron::Schedule::from_str(&cron.expression);

    match schedule {
        Ok(sched) => {
            let next_runs: Vec<DateTime<Utc>> = sched.upcoming(Utc).take(10).collect();
            let next_runs_as_strings: Vec<String> = next_runs
                .iter()
                .map(|x: &DateTime<Utc>| x.to_rfc3339())
                .collect();
            Json(NextRuns {
                timezone: "UTC".to_string(),
                next_runs: next_runs_as_strings,
            })
        }
        Err(e) => {
            println!("Error: {:?}", e);
            return Json(NextRuns {
                timezone: "UTC".to_string(),
                next_runs: vec![],
            });
        }
    }
}

async fn get_all_schedules<T: SchedulerRepo>(State(state): State<T>) -> Json<Vec<Schedule>> {
    let schedules = state.get_all().await;
    Json(schedules)
}

async fn create_a_schedule<T: SchedulerRepo>(
    State(state): State<T>,
    Json(schedule): Json<Schedule>,
) -> Json<i64> {
    let id = state.create(schedule).await;
    Json(id)
}

async fn get_schedule<T: SchedulerRepo>(
    State(state): State<T>,
    Path(id): Path<i64>,
) -> Json<Option<Schedule>> {
    let schedule = state.get(id).await;
    Json(schedule)
}

async fn update_schedule<T: SchedulerRepo>(
    State(state): State<T>,
    Path(id): Path<i64>,
    Json(schedule): Json<Schedule>,
) -> Json<()> {
    state.update(id, schedule).await;
    Json(())
}

async fn delete_schedule<T: SchedulerRepo>(
    State(state): State<T>,
    Path(id): Path<i64>,
) -> Json<()> {
    state.delete(id).await;
    Json(())
}

async fn health() -> Json<Health> {
    let health_stats = Health {
        status: "OK".to_string(),
    };
    Json(health_stats)
}

async fn config(Extension(config): Extension<Arc<AppConfig>>) -> Json<AppConfig> {
    Json(config.as_ref().clone())
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Health {
    status: String,
}

// The schedule model. This is what the API will be working with.
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Schedule {
    pub id: u64,
    pub client_id: u64,
    pub filename: String,
    pub vendor: String,
    pub target: String,
    pub cron_expression: String,
    pub next_run: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Cron {
    expression: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct NextRuns {
    timezone: String,
    next_runs: Vec<String>,
}

// The functions that the state/storage of schedules should implement.
// We can use this to mock a service for testing so we don't have to hit a database when testing.
#[async_trait]
pub trait SchedulerRepo: Send + Sync + Clone + 'static {
    async fn get_all(&self) -> Vec<Schedule>;
    async fn create(&self, schedule: Schedule) -> i64;
    async fn get(&self, id: i64) -> Option<Schedule>;
    async fn update(&self, id: i64, schedule: Schedule) -> ();
    async fn delete(&self, id: i64) -> ();
}

// Implement the SchedulerRepo trait for the SchedulerRepoPostgres struct.
#[async_trait]
impl SchedulerRepo for SchedulerRepoPostgres {
    async fn get_all(&self) -> Vec<Schedule> {
        let schedules = sqlx::query!("SELECT * FROM schedules")
            .fetch_all(&self.pool)
            .await
            .unwrap();
        schedules
            .into_iter()
            .map(|schedule| Schedule {
                id: schedule.id as u64,
                client_id: schedule.client_id as u64,
                filename: schedule.filename,
                vendor: schedule.vendor,
                target: schedule.target,
                cron_expression: schedule.cron_expression,
                next_run: Some(schedule.next_run),
            })
            .collect()
    }

    async fn create(&self, schedule: Schedule) -> i64 {
        let mut next_run = None;
        if schedule.next_run.is_none() {
            let cron_schedule = cron::Schedule::from_str(&schedule.cron_expression).unwrap();
            next_run = Some(cron_schedule.upcoming(Utc).next().unwrap());
        }
        sqlx::query!(
            "INSERT INTO schedules (client_id, filename, vendor, target, cron_expression, next_run) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            schedule.client_id as i64,
            schedule.filename,
            schedule.vendor,
            schedule.target,
            schedule.cron_expression,
            next_run
        )
        .fetch_one(&self.pool)
        .await
        .unwrap()
        .id
    }

    async fn get(&self, _id: i64) -> Option<Schedule> {
        todo!()
    }

    async fn update(&self, _id: i64, _schedule: Schedule) -> () {
        todo!()
    }

    async fn delete(&self, _id: i64) -> () {
        sqlx::query!("DELETE FROM schedules WHERE id = $1", _id)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct SchedulerRepoPostgres {
    pool: Pool<Postgres>,
}

// Configure the Postgres connection pool.
impl SchedulerRepoPostgres {
    pub async fn new() -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(&env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        Self { pool }
    }
}

// Helper function to get the Postgres connection pool.
pub async fn get_pool() -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(16)
        .connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap()
}

// Helper function to get a Duration in order to how often the scheduler should check for schedules that are due to run.
fn get_env_var_as_int(key: &str, default: u64) -> time::Duration {
    let value = env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default);
    time::Duration::from_secs(value)
}

// This function gets all schedules that are due to run and sends them to the worker function using an mpsc channel.
pub async fn enqueue_schedules(pool: Pool<Postgres>, sender: Sender<Schedule>) {
    let run_mode = get_env_var_as_int("APP_SCHEDULE_CHECK_INTERVAL_IN_SECONDS", 60);
    let mut interval = time::interval(run_mode);
    loop {
        interval.tick().await;
        let now = Utc::now();
        let schedules = sqlx::query!("SELECT id, client_id, filename, vendor, target, cron_expression, next_run FROM schedules WHERE next_run <= $1", now).fetch_all(&pool).await.unwrap();

        for schedule in schedules {
            // Create a Schedule struct from the database row because we can't call clone() a record.
            let s = Schedule {
                id: schedule.id.clone() as u64,
                client_id: schedule.client_id.clone() as u64,
                filename: schedule.filename.clone(),
                vendor: schedule.vendor.clone(),
                target: schedule.target.clone(),
                cron_expression: schedule.cron_expression.clone(),
                next_run: Some(schedule.next_run.clone()),
            };

            // Send the schedule to be processed
            sender.send(s).await.unwrap();

            // Update the next_run based on the cron expression
            let cron_schedule = cron::Schedule::from_str(&schedule.cron_expression).unwrap();
            let next_run = cron_schedule.upcoming(Utc).next().unwrap();
            sqlx::query!(
                "UPDATE schedules SET next_run = $1 WHERE id = $2",
                next_run,
                schedule.id as i64
            )
            .execute(&pool)
            .await
            .unwrap();
        }
    }
}

// Process schedules sent by enqueue_schedules that are due to run.
// Currently we just print the message but here's where we invoke the
// Vendor Communication API with the schedule object.
pub async fn process_schedules(mut scheduler: Receiver<Schedule>) {
    while let Some(schedule) = scheduler.recv().await {
        println!("Processing schedule: {:?}", schedule);
    }
}

// Simple test to make sure we can connect to the database using sqlx.
#[tokio::test]
async fn select_one_plus_one() {
    let _pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _sum: i32 = sqlx::query!("SELECT 1 + 1 AS sum")
        .fetch_one(&_pool)
        .await
        .unwrap()
        .sum
        .unwrap();

    println!("_sum: {}", _sum);

    assert_eq!(_sum, 2);
}
