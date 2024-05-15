pub mod mock;

use crate::mock::MockSchedulerRepo;
use axum::{
    body::Body,
    http::{Request, StatusCode}, //Json, Router,
};
use http_body_util::BodyExt;
use rust_scheduler_service::{
    app::{create_scheduler_app, Schedule, SchedulerRepo},
    settings::AppConfig,
};
//use std::sync::Arc;
use tower::ServiceExt; // for `app.oneshot()`

#[tokio::test]
async fn test_get_all_schedules() {
    let mock_repo = MockSchedulerRepo::default();
    let app_config = AppConfig::new().unwrap();
    let app = create_scheduler_app(mock_repo.clone(), app_config);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/schedules")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let schedules: Vec<Schedule> = serde_json::from_slice(&body).unwrap();
    assert!(schedules.is_empty());
}

#[tokio::test]
async fn test_create_a_schedule() {
    let mock_repo = MockSchedulerRepo::default();
    let app_config = AppConfig::new().unwrap();
    let app = create_scheduler_app(mock_repo.clone(), app_config);

    let new_schedule = Schedule {
        id: 0,
        client_id: 1,
        filename: "test.txt".to_string(),
        vendor: "vendor".to_string(),
        target: "target".to_string(),
        cron_expression: "0 5 * * *".to_string(),
        next_run: None,
    };

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/schedules")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&new_schedule).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let id: i64 = serde_json::from_slice(&body).unwrap();
    assert_eq!(id, 1);

    let schedules = mock_repo.get_all().await;
    assert_eq!(schedules.len(), 1);
    assert_eq!(schedules[0].filename, "test.txt");
}

#[tokio::test]
async fn test_get_schedule() {
    let mock_repo = MockSchedulerRepo::default();
    let new_schedule = Schedule {
        id: 1,
        client_id: 1,
        filename: "test.txt".to_string(),
        vendor: "vendor".to_string(),
        target: "target".to_string(),
        cron_expression: "0 5 * * *".to_string(),
        next_run: None,
    };

    mock_repo.create(new_schedule).await;

    let app_config = AppConfig::new().unwrap();
    let app = create_scheduler_app(mock_repo, app_config);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/schedules/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let schedule: Option<Schedule> = serde_json::from_slice(&body).unwrap();
    assert!(schedule.is_some());
    assert_eq!(schedule.unwrap().filename, "test.txt");
}

#[tokio::test]
async fn test_update_schedule() {
    todo!("implement test_update_schedule")
}

#[tokio::test]
async fn test_delete_schedule() {
    todo!("implement test_delete_schedule")
}

#[tokio::test]
async fn test_get_next_runs() {
    todo!("implement test_get_delete_schedule")
}
