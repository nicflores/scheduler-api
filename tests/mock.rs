use async_trait::async_trait;
use rust_scheduler_service::app::{Schedule, SchedulerRepo};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct MockSchedulerRepo {
    schedules: Arc<Mutex<Vec<Schedule>>>,
}

#[async_trait]
impl SchedulerRepo for MockSchedulerRepo {
    async fn get_all(&self) -> Vec<Schedule> {
        let schedules = self.schedules.lock().unwrap();
        schedules.clone()
    }

    async fn create(&self, schedule: Schedule) -> i64 {
        let mut schedules = self.schedules.lock().unwrap();
        let id = schedules.len() as i64 + 1;
        schedules.push(Schedule {
            id: id as u64,
            ..schedule
        });
        id
    }

    async fn get(&self, id: i64) -> Option<Schedule> {
        let schedules = self.schedules.lock().unwrap();
        schedules.iter().find(|s| s.id == id as u64).cloned()
    }

    async fn update(&self, id: i64, schedule: Schedule) {
        let mut schedules = self.schedules.lock().unwrap();
        if let Some(existing_schedule) = schedules.iter_mut().find(|s| s.id == id as u64) {
            *existing_schedule = schedule;
        }
    }

    async fn delete(&self, id: i64) {
        let mut schedules = self.schedules.lock().unwrap();
        schedules.retain(|s| s.id != id as u64);
    }
}
