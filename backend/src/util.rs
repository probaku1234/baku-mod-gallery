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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chrono_dt_from_string() {
        let dt = get_chrono_dt_from_string("2022-03-14T05:23:49.000+00:00".to_string());

        assert_eq!(dt.timestamp(), 1647235429);
    }

    #[test]
    fn test_get_chrono_dt_from_string_with_invalid_string() {
        let dt = get_chrono_dt_from_string("qweqwe".to_string());

        assert_eq!(dt.timestamp(), 0);
    }

    #[test]
    fn test_convert_to_rfc3999_string() {
        let date_string = convert_to_rfc3999_string("2022-03-14T05:23:49.000+00:00".to_string());

        assert_eq!(date_string, "2022-03-14T05:23:49Z");
    }

    #[test]
    fn test_convert_to_rfc3999_string_with_invalid_string() {
        let date_string = convert_to_rfc3999_string("asdfasdf".to_string());

        assert_eq!(date_string, "1970-01-01T00:00:00Z");
    }
}
