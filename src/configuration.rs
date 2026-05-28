use secrecy::{ExposeSecret, SecretBox};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretBox<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password.expose_secret(), self.host, self.port, self.database_name
        )))
    }

    pub fn connection_string_without_db(&self) -> SecretBox<String> {
        SecretBox::new(Box::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        )))
    }
}


pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        // 从名为“configuration”的文件中添加配置值。
        // 它将查找任何具有扩展名的顶级文件
        // `config` 知道如何解析：yaml、json 等。
        .add_source(config::File::with_name("configuration"))
        .build()?;

    // 尝试将其读取的配置值转换成
    // 我们的设置类型
    settings.try_deserialize()
}
