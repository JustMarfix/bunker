use crate::config::Modes::{BodySubstr, UrlSubstr};
use crate::config::Service;
use async_trait::async_trait;
use pingora::prelude::*;
use std::str;
use std::sync::Arc;

#[derive(Clone)]
pub struct LB {
    pub upstream: Arc<LoadBalancer<RoundRobin>>,
    pub config: Service
}

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();
    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream = self.upstream
            .select(b"", 256)
            .unwrap();

        let peer = Box::new(HttpPeer::new(upstream, false, "bunker.proxy".to_string()));
        Ok(peer)
    }

    async fn proxy_upstream_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        for filter in &self.config.filters {
             if filter.mode == BodySubstr && _session.is_body_empty() {
                 match _session.read_request_body().await {
                     Ok(body_opt) => {
                         if filter.aho_corasick.is_match(str::from_utf8(&body_opt.unwrap()).unwrap()) {
                             _session.respond_error(403).await?;
                             return Ok(false)
                         }
                     }
                     Err(_) => { }
                 }
             } else if filter.mode == UrlSubstr {
                 if filter.aho_corasick.is_match(_session.req_header().uri.path()) {
                     _session.respond_error(403).await?;
                     return Ok(false)
                 }
             }
         }
        Ok(true)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("X-Forwarded-IP", _session.client_addr().unwrap().as_inet().unwrap().ip().to_string())?;
        Ok(())
    }
}

