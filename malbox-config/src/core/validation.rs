use super::Config;
use crate::ConfigError;

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validate_database()?;
        self.validate_http()?;
        self.validate_analysis()?;
        Ok(())
    }

    fn validate_database(&self) -> Result<(), ConfigError> {
        if self.database.password.is_none() && self.database.password_env.is_none() {
            return Err(ConfigError::InvalidValue {
                field: "database".into(),
                message: "Either password or password_env must be set".into(),
            });
        }

        if let Some(env_var) = &self.database.password_env {
            std::env::var(env_var).map_err(|_| ConfigError::EnvVarNotSet(env_var.clone()))?;
        }
        Ok(())
    }

    fn validate_http(&self) -> Result<(), ConfigError> {
        if self.http.tls_enabled {
            if self.http.cert_path.is_none() || self.http.key_path.is_none() {
                return Err(ConfigError::InvalidValue {
                    field: "http".into(),
                    message: "TLS is enabled but cert_path or key_path is missing".into(),
                });
            }
        }
        Ok(())
    }

    fn validate_analysis(&self) -> Result<(), ConfigError> {
        if self.analysis.max_vms == 0 {
            return Err(ConfigError::InvalidValue {
                field: "analysis.max_vms".into(),
                message: "max_vms must be greater than 0".into(),
            });
        }
        Ok(())
    }
}
