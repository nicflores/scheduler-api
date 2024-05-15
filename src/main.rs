mod app;
mod settings;

use crate::app::SchedulerRepoPostgres;
use crate::app::{enqueue_schedules, get_pool, process_schedules};
use std::future::IntoFuture;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // app config
    let cfg = settings::AppConfig::new().unwrap();

    // logging
    // Once we setup Honeycomb we can uncomment this line
    //init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()) // Logs to stdout
        // .with(tracing_opentelemetry::layer()) // Sends traces to Honeycomb.io enable this later
        .try_init()
        .unwrap();

    // api server
    let app = app::create_scheduler_app(SchedulerRepoPostgres::new().await, cfg);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(listener, app).into_future(); //.await .unwrap();

    // process schedules
    let (sender, receiver): (Sender<app::Schedule>, Receiver<app::Schedule>) = mpsc::channel(100);

    // Setup the up the scheduler checker and worker
    let pool = get_pool().await;
    let scheduler_task = enqueue_schedules(pool, sender);
    let schedule_worker = process_schedules(receiver);

    // Run the server and scheduler concurrently
    tokio::select! {
        _ = server => eprintln!("Server error"),
        _ = scheduler_task => eprintln!("Scheduler error"),
        _ = schedule_worker => eprintln!("Worker error"),
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Shutting down...");
        },
    }
}
