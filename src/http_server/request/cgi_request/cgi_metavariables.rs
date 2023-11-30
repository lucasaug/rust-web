use std::{collections::HashMap, ffi::OsStr};

#[derive(
    strum_macros::Display, strum_macros::IntoStaticStr, strum_macros::AsRefStr, Eq, Hash, PartialEq,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum CGIMetavariable {
    AuthType,
    ContentLength,
    ContentType,
    GatewayInterface,
    PathInfo,
    PathTranslated,
    QueryString,
    RemoteAddr,
    RemoteHost,
    RemoteIdent,
    RemoteUser,
    RequestMethod,
    ScriptName,
    ServerName,
    ServerPort,
    ServerProtocol,
    ServerSoftware,
}

impl AsRef<OsStr> for CGIMetavariable {
    fn as_ref(&self) -> &OsStr {
        let a: &str = self.as_ref();
        OsStr::new(a)
    }
}

pub type CGIMetavariableMap = HashMap<CGIMetavariable, String>;
