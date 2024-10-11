mod config;
mod proxy;

use log::info;
use pingora::prelude::*;
use pretty_env_logger;
use std::env::set_var;
use std::sync::Arc;
use aho_corasick::AhoCorasick;
use crate::config::*;
use crate::proxy::*;

fn main() {
    set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    let config_file: Config = load_config("./config.yaml").unwrap();
    info!("Config loaded successfully");

    let mut my_server = Server::new(None).unwrap();
    my_server.bootstrap();
    info!("Pingora server loaded successfully");

    for mut service in config_file.services {
        for filter in &mut service.filters {
            filter.aho_corasick = AhoCorasick::new(&[&filter.substr]).unwrap()
        }

        let upstream = LoadBalancer::try_from_iter([&service.upstream]).unwrap();
        let balancer: LB = LB{upstream: Arc::new(upstream), config: service};
        let mut lb = http_proxy_service(&my_server.configuration, balancer.clone());

        lb.add_tcp(&balancer.config.listen);
        my_server.add_service(lb);

        info!("Loaded proxy configuration for {} ({} filters)", &balancer.config.upstream, &balancer.config.filters.iter().count())
    }

    my_server.run_forever()
}
