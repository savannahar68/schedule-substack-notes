use std::path::PathBuf;

pub struct Config {
    pub port: u16,
    pub data_dir: PathBuf,
    pub encryption_key_override: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6894u16);

        let data_dir = std::env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        let encryption_key_override = std::env::var("ENCRYPTION_KEY").ok();

        Self {
            port,
            data_dir,
            encryption_key_override,
        }
    }

    pub fn database_url(&self) -> String {
        format!("sqlite:{}/scheduler.db?mode=rwc", self.data_dir.display())
    }

    pub fn encryption_key_path(&self) -> PathBuf {
        self.data_dir.join("encryption.key")
    }

    /// Load encryption key from env, or from file, or generate and persist a new one.
    pub fn load_or_create_encryption_key(&self) -> [u8; 32] {
        if let Some(ref key_hex) = self.encryption_key_override {
            let bytes = hex::decode(key_hex).expect("ENCRYPTION_KEY must be valid hex");
            assert_eq!(bytes.len(), 32, "ENCRYPTION_KEY must be 32 bytes (64 hex chars)");
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return key;
        }

        let key_path = self.encryption_key_path();
        if key_path.exists() {
            let contents = std::fs::read_to_string(&key_path).expect("Failed to read encryption.key");
            let bytes = hex::decode(contents.trim()).expect("encryption.key contains invalid hex");
            assert_eq!(bytes.len(), 32, "encryption.key must contain 32 bytes (64 hex chars)");
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            return key;
        }

        // Generate new key
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        let key_hex = hex::encode(key);
        std::fs::write(&key_path, &key_hex).expect("Failed to write encryption.key");
        tracing::info!("Generated new encryption key at {}", key_path.display());
        key
    }
}
