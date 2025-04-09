use std::fs;
use std::error::Error;
use tokio_tungstenite::Connector;

pub fn get_tls_connector() -> Result<Connector, Box<dyn Error>> {
    let cert_file = fs::read("/home/huishi/work/service/certs/localhost.crt")?;
    let cert = native_tls::Certificate::from_pem(&cert_file)?;
    let tls_connector = native_tls::TlsConnector::builder()
        .add_root_certificate(cert)
        .build()?;
    Ok(Connector::NativeTls(tls_connector))
}
