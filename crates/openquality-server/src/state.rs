use std::sync::Arc;

use openquality_auth::JwtManager;
use openquality_core::alert::AlertManager;
use openquality_core::store::{InMemoryStore, Store};
use openquality_scheduler::Scheduler;
use openquality_store::pg::PgStore;
use tokio::sync::RwLock;

pub struct AppState {
    pub store: Arc<dyn Store>,
    pub alert_manager: RwLock<AlertManager>,
    pub jwt_manager: JwtManager,
}

impl AppState {
    #[expect(dead_code)]
    pub fn new(store: Box<dyn Store>, alert_manager: AlertManager, jwt_secret: &[u8]) -> Self {
        Self {
            store: Arc::from(store),
            alert_manager: RwLock::new(alert_manager),
            jwt_manager: JwtManager::new(jwt_secret, 24),
        }
    }

    pub async fn new_with_defaults() -> Self {
        let database_url = std::env::var("DATABASE_URL").ok();
        let store: Arc<dyn Store> = if let Some(url) = &database_url {
            match PgStore::connect(url).await {
                Ok(pg) => {
                    tracing::info!("Connected to PostgreSQL, running migrations...");
                    if let Err(e) = pg.migrate().await {
                        tracing::warn!("Migration error (may be ok): {}", e);
                    }
                    Arc::new(pg)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to connect to PostgreSQL ({}), falling back to InMemoryStore",
                        e
                    );
                    Arc::new(InMemoryStore::new())
                }
            }
        } else {
            tracing::info!("No DATABASE_URL set, using InMemoryStore");
            Arc::new(InMemoryStore::new())
        };

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "openquality-dev-secret-change-in-production".to_string());
        if jwt_secret == "openquality-dev-secret-change-in-production" {
            tracing::warn!("Using default JWT secret — set JWT_SECRET env var in production");
        }

        let mut alert_mgr = AlertManager::new(store.clone());
        alert_mgr.add_channel(Box::new(
            openquality_core::alert::channel::StdoutAlertChannel::new("stdout"),
        ));

        Self {
            store: store.clone(),
            alert_manager: RwLock::new(alert_mgr),
            jwt_manager: JwtManager::new(jwt_secret.as_bytes(), 24),
        }
    }

    #[expect(dead_code)]
    pub fn start_scheduler(&self) {
        let scheduler = Scheduler::new(
            self.store.clone(),
            Arc::new(RwLock::new(AlertManager::new(self.store.clone()))),
        );
        tokio::spawn(async move {
            scheduler.start().await;
        });
    }
}
