use chrono::Utc;
use mongodb::bson::DateTime;

pub fn get_chrono_dt_from_string(date_string: String) -> chrono::DateTime<Utc> {
    let chrono_dt: chrono::DateTime<Utc> = date_string
        .parse()
        .unwrap_or("1970-01-01T09:00:00+09:00".parse().unwrap());

    chrono_dt
}

pub fn convert_to_rfc3999_string(date_string: String) -> String {
    let chrono_dt: chrono::DateTime<Utc> = date_string
        .parse()
        .unwrap_or("1970-01-01T09:00:00+09:00".parse().unwrap());
    let bson_dt = DateTime::from_chrono(chrono_dt);

    bson_dt
        .try_to_rfc3339_string()
        .unwrap_or("1970-01-01T09:00:00+09:00".to_string())
}
