#[cfg(test)]
pub mod test_util {
    use std::net::UdpSocket;
    use run_script::run_script;
    use testcontainers::{GenericImage, RunnableImage};

    pub fn generate_port_number() -> u16 {
        let address = "0.0.0.0:0";
        let socket = UdpSocket::bind(address).expect("Cannot bind to socket");
        let local_addr = socket.local_addr().expect("Cannot get local address");
        local_addr.port()
    }

    pub fn get_mongo_image(&port: &u16) -> RunnableImage<GenericImage> {
        let image = GenericImage::new(
            "mongo".to_string(),
            "5.0.6".to_string(),
        );
        RunnableImage::from(image).with_mapped_port((port, 27017))
    }

    pub fn populate_test_data(&port: &u16) {
        let formatted_command = format!(r#" bash ./tests/test_data/import.sh {} {}"#, "0.0.0.0", port);
        run_script!(formatted_command).expect("Cannot seed MongoDB data");
    }

    pub fn get_db_connection_uri(&port: &u16) -> String {
        format!("mongodb://{}:{}", "0.0.0.0", port)
    }
}