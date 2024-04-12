use anyhow::Result;
use redis::Commands;
use uuid::Uuid;

const JOB_ID_KEY: &str = "job_id";

pub fn create_sync_job(redis: redis::Client) -> Result<()> {
    let mut con = redis.get_connection()?;
    con.set(JOB_ID_KEY, Uuid::new_v4().simple().to_string())?;

    Ok(())
}

pub fn delete_sync_job(redis: redis::Client) -> Result<i64> {
    let mut con = redis.get_connection()?;
    let deleted_key_num: i64 = con.del(JOB_ID_KEY)?;

    Ok(deleted_key_num)
}

pub fn check_sync_job_exists(redis: redis::Client) -> Result<bool> {
    let mut con = redis.get_connection()?;

    let job_id: Option<String> = con.get(JOB_ID_KEY)?;

    Ok(job_id.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::test_util::{get_redis_connection_uri, get_redis_image};
    use testcontainers_modules::redis::REDIS_PORT;
    use testcontainers_modules::testcontainers::clients;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_create_sync_job() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let result = create_sync_job(redis_client);

        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_sync_job_when_no_job_exist() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let result = delete_sync_job(redis_client);

        assert!(result.is_ok());

        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_delete_sync_job_when_job_exist() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let mut con = redis_client.clone().get_connection().unwrap();
        let _: () = con.set(JOB_ID_KEY, "123123123").unwrap();

        let result = delete_sync_job(redis_client);

        assert!(result.is_ok());

        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_check_sync_job_exist_when_no_job_exist() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let result = check_sync_job_exists(redis_client);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_check_sync_job_exist_when_job_exist() {
        let docker = clients::Cli::default();
        let redis_img = get_redis_image();
        let redis_node = docker.run(redis_img);

        let redis_uri = get_redis_connection_uri(&redis_node.get_host_port_ipv4(REDIS_PORT));
        let redis_client = redis::Client::open(redis_uri.as_ref()).unwrap();

        let mut con = redis_client.clone().get_connection().unwrap();
        let _: () = con.set(JOB_ID_KEY, "123123123").unwrap();

        let result = check_sync_job_exists(redis_client);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
