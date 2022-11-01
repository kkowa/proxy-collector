//! Context module for handlers.

use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc}};

use super::Proxy;
use crate::auth::Credentials;

/// Shared state of application context across handlers.
#[derive(Clone, Debug)]
pub struct Flow {
    /// Current flow's numeric sequence ID.
    id: u64,

    /// Parent app which current context have derive from.
    app: Arc<Proxy>,

    /// Incoming request source address.
    client: SocketAddr,

    /// Proxy authentication credentials. First passed auth credentials will be set if multiple auth backends set.
    auth: Option<Credentials>,
}

impl Flow {
    /// Create new flow.
    pub fn new(proxy: Proxy, client: SocketAddr) -> Self {
        Self {
            id: proxy.counter.fetch_add(1, Ordering::SeqCst),
            app: Arc::new(proxy),
            client,
            auth: None,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn app(&self) -> Arc<Proxy> {
        Arc::clone(&self.app)
    }

    pub fn client(&self) -> &SocketAddr {
        &self.client
    }

    pub fn auth(&self) -> Option<&Credentials> {
        self.auth.as_ref()
    }

    pub fn auth_mut(&mut self) -> &mut Option<Credentials> {
        &mut self.auth
    }
}

#[cfg(test)]
mod tests {}
