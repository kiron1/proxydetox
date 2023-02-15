use std::{net::SocketAddr, sync::Arc};

use dnsdetox::{dns, doh};
use futures_util::StreamExt;
use http::Uri;
use paclib::Proxy;

mod options;

struct Upstream {
    primary: dns::Client,
    secondary: doh::Client,
}

impl Upstream {
    fn new(primary_addr: SocketAddr, secondary_uri: Uri, proxy: Proxy) -> Self {
        let primary = dns::Client::new(primary_addr);
        let secondary = doh::Client::new(secondary_uri, proxy);
        Self { primary, secondary }
    }

    async fn process(&self, from: dns::ClientRef, data: Vec<u8>) -> dnsdetox::error::Result<()> {
        let resp = self.primary.request(&data).await?;

        let pkt = dns_parser::Packet::parse(&resp)?;

        let resp = match (pkt.questions.is_empty(), pkt.answers.is_empty()) {
            (false, true) => {
                log::info!("ask secondary DNS server");

                self.secondary.request(data).await?
            }
            _ => resp,
        };

        from.reply(&resp).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = options::Options::load();

    let upstream = Upstream::new(options.primary, options.secondary, options.proxy);
    let upstream = Arc::new(upstream);

    let server = dns::Server::new(options.port);
    let mut server = server.serve().await?;

    while let Some(query) = server.next().await {
        match query {
            Ok((from, data)) => {
                let up = Arc::clone(&upstream);
                log::debug!("query from: {}", &from.remote_addr());
                tokio::spawn(async move {
                    if let Err(cause) = up.process(from, data).await {
                        log::error!("process error: {}", &cause);
                    }
                });
            }
            Err(ref cause) => log::error!("server error: {}", &cause),
        }
    }

    Ok(())
}
