use zenoh_proto::SessionError;

use crate::{
    api::session::Session,
    config::ZSessionConfig,
};

/// Liveliness token: allows manually sending heartbeat puts on a key expression.
pub struct LivelinessToken<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'a Session<'res, Config>,
    ke: &'a zenoh_proto::keyexpr,
}

impl<'a, 'res, Config> LivelinessToken<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub(crate) fn new(session: &'a Session<'res, Config>, ke: &'a zenoh_proto::keyexpr) -> Self {
        Self { session, ke }
    }

    /// Send a heartbeat (empty payload Put) on the token's key expression.
    pub async fn heartbeat(&self) -> core::result::Result<(), SessionError> {
        self.session.put(self.ke, b"").finish().await
    }

    pub fn keyexpr(&self) -> &zenoh_proto::keyexpr {
        self.ke
    }
}

pub struct LivelinessBuilder<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    session: &'a Session<'res, Config>,
}

impl<'a, 'res, Config> LivelinessBuilder<'a, 'res, Config>
where
    Config: ZSessionConfig,
{
    pub fn declare_token(
        self,
        ke: &'a zenoh_proto::keyexpr,
    ) -> LivelinessToken<'a, 'res, Config> {
        LivelinessToken::new(self.session, ke)
    }
}

impl<'res, Config> Session<'res, Config>
where
    Config: ZSessionConfig,
{
    /// Access the liveliness API for this session.
    pub fn liveliness(&self) -> LivelinessBuilder<'_, 'res, Config> {
        LivelinessBuilder { session: self }
    }
}
