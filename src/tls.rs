use std::fs;
use std::error::Error;
use tokio_tungstenite::Connector;

pub fn get_tls_connector(cert_file_path: &str) -> Result<Connector, Box<dyn Error>> {
    let cert_file = fs::read(cert_file_path)?;
    let cert = native_tls::Certificate::from_pem(&cert_file)?;
    let tls_connector = native_tls::TlsConnector::builder()
        .add_root_certificate(cert)
        .build()?;
    Ok(Connector::NativeTls(tls_connector))
}
