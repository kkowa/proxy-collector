//! Main binary for use by kkowa application system.

mod auth;
mod reporter;

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use lib::{http::Uri, Proxy, Web};
use tracing::{debug, error, info, warn, Level};

use self::reporter::{Processor, Reporter};

macro_rules! arg_env {
    ($name:literal) => {
        concat!("APP_", $name)
    };
}

type Client = hyper::Client<hyper::client::HttpConnector>;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Config {
    /// Host address the proxy server listen to
    #[clap(long, env = arg_env!("HOST"), default_value = "0.0.0.0")]
    host: String,

    /// Port number the server will listen to
    #[clap(long, env = arg_env!("PORT"), default_value = "1080")]
    port: u16,

    /// Host address the proxy server listen to
    #[clap(long, env = arg_env!("WEB_HOST"), default_value = "0.0.0.0")]
    web_host: String,

    /// Port number the server will listen to
    #[clap(long, env = arg_env!("WEB_PORT"), default_value = "8080")]
    web_port: u16,

    /// Set tracing verbosity. Available values are "trace", "debug", "info", "warn". Cases are ignored.
    #[clap(long, env = arg_env!("VERBOSITY"), default_value = "info")]
    verbosity: String,

    /// Core server base URL.
    #[clap(long, env = arg_env!("SERVER"))]
    server: Option<Uri>,

    /// File or directory path for processor definition file(s). If path is directory, try to load all YAML files in
    /// directory as processor. If not set, load default processors.
    #[clap(short, long, env = arg_env!("PROCESSOR"))]
    processor: Option<PathBuf>,
}

pub fn init_tracing(max_level: Level) {
    let subscriber = tracing_subscriber::fmt().with_max_level(max_level).finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global default tracing subscriber");

    info!("initialized tracing with max level `{max_level}`");
}

#[tokio::main]
async fn main() {
    // Parse CLI args
    let config = Config::parse();

    // Initialize tracing
    let level = match config.verbosity.to_lowercase().as_ref() {
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        invalid => panic!("invalid verbosity {invalid}"),
    };
    init_tracing(level);

    // Load processor(s)
    let processor_defs = match config.processor {
        Some(path) => {
            if !path.exists() {
                panic!("processor path {path:?} is neither directory or file, may not exists")
            }

            if path.is_dir() {
                debug!("looking for processor defs from directory {path:?}");
                let entries = path
                    .read_dir()
                    .expect("failed to read directory")
                    .into_iter()
                    .map(|entry| entry.expect("failed to read entry").path());

                entries
                    .filter(|p| {
                        if let Some(ext) = p.extension() {
                            ext == "yaml" || ext == "yml"
                        } else {
                            false
                        }
                    })
                    .collect()
            } else {
                debug!("specified processor path is single file: {path:?}");

                vec![path]
            }
        }
        None => {
            // Load default processors
            vec![
                // include_str!("./processors/<NAME>.yaml")
            ]
        }
    };

    let processors = processor_defs
        .into_iter()
        .inspect(|p| debug!("loading processor def {p:?}"))
        .map(|p| Processor::from_file(p).expect("failed to load file {path} as processor"))
        .collect();

    debug!("loaded processors: {processors:?}");

    // Run app
    let proxy_addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("failed to parse socket address");

    let web_addr: SocketAddr = format!("{}:{}", config.web_host, config.web_port)
        .parse()
        .expect("failed to parse socket address");

    // TODO: Support CLI arguments for static proxy auth credentials
    let proxy = Proxy::new(
        "proxy",
        Client::default(),
        vec![Box::new(self::auth::ServerAuth::new(
            config.server.clone().map(|u| {
                Uri::builder()
                    .scheme(u.scheme_str().unwrap())
                    .authority(u.authority().unwrap().to_string())
                    .path_and_query("")
                    .build()
                    .unwrap()
            }),
        ))],
        vec![Box::new(Reporter::new(
            config.server.clone().map(|u| {
                Uri::builder()
                    .scheme(u.scheme_str().unwrap())
                    .authority(u.authority().unwrap().to_string())
                    .path_and_query("")
                    .build()
                    .unwrap()
            }),
            processors,
        ))],
    );

    let web = Web::new();

    warn!("proxy listening on {}", proxy_addr);
    warn!("web listening on {}", web_addr);
    if let Err(e) = tokio::try_join!(proxy.run(&proxy_addr), web.run(&web_addr)) {
        error!("error occurred from server: {e}");
    }
}

#[cfg(test)]
mod tests {}
