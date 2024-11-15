use std::net::{Ipv4Addr, Ipv6Addr};

use crate::errors::UrlParseResult;
use crate::UrlParseError;
use url::Url;

fn is_locahost(uri: &Url) -> bool {
    if let Some(host) = uri.host() {
        match host {
            url::Host::Domain(domain) => domain == "localhost",
            url::Host::Ipv4(ip) => ip == Ipv4Addr::LOCALHOST,
            url::Host::Ipv6(ip) => ip == Ipv6Addr::LOCALHOST,
        }
    } else {
        false
    }
}

pub fn check_uri(uri: &str) -> UrlParseResult<Url> {
    let uri = Url::parse(uri);

    let uri = match uri {
        Ok(res) => res,
        Err(err) => return Err(UrlParseError::Parser(err)),
    };

    if uri.scheme() != "https" && !is_locahost(&uri) {
        return Err(UrlParseError::NotHttps);
    }

    Ok(uri)
}
