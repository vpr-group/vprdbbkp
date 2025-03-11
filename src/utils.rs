use chrono::Utc;
use uuid::Uuid;

pub fn get_backup_key(prefix: &str, db_type: &str, db_name: &str) -> String {
    let now = Utc::now();
    let date_str = now.format("%Y-%m-%d-%H%M%S");
    let uuid_string = Uuid::new_v4().to_string();
    let uuid = uuid_string.split('-').next().unwrap_or("backup");

    format!(
        "{}/{}/{}-{}-{}.gz",
        prefix, db_type, db_name, date_str, uuid
    )
}
