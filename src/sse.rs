use eventsource_client::*;
use futures_util::StreamExt;
use url::Url;

enum UrlScheme {
    Https,
    Http,
}

pub struct ServerEvents {
    client_builder: ClientBuilder,
    scheme: UrlScheme,
}

impl ServerEvents {
    pub fn new(url: &str) -> Option<Self> {
        let client_builder_result = ClientBuilder::for_url(url);
        let scheme = match Url::parse(url) {
            Ok(url) => match url.scheme() {
                "https" => UrlScheme::Https,
                "http" => UrlScheme::Http,
                _ => return None,
            },
            Err(_) => return None,
        };

        match client_builder_result {
            Ok(client_builder) => Some(ServerEvents {
                client_builder,
                scheme,
            }),
            Err(_) => None,
        }
    }

    pub async fn listen(
        self,
        stream_event: impl Fn(String, Option<String>),
        stream_err: impl Fn(Error),
        keep_alive_friendly: bool,
    ) {
        self.stream(keep_alive_friendly)
            .for_each(|event| {
                match event {
                    Ok((event_type, maybe_data)) => stream_event(event_type, maybe_data),
                    Err(x) => stream_err(x),
                }
                futures_util::future::ready(())
            })
            .await
    }

    /**
     * Create a stream of server-sent events.
     */
    pub fn stream(
        self,
        keep_alive_friendly: bool,
    ) -> std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<(String, Option<String>)>> + Send>>
    {
        let client: Box<dyn Client> = match self.scheme {
            UrlScheme::Https => Box::new(self.client_builder.build()),
            UrlScheme::Http => Box::new(self.client_builder.build_http()),
        };
        return Box::pin(client.stream().filter_map(move |event| async move {
            match event {
                Ok(SSE::Event(ev)) => {
                    if ev.event_type == "keep-alive" && !keep_alive_friendly {
                        return None;
                    }

                    if ev.data == "null" {
                        return Some(Ok((ev.event_type, None)));
                    }

                    return Some(Ok((ev.event_type, Some(ev.data))));
                }
                Ok(SSE::Comment(_)) => return None,
                Ok(SSE::Connected(_)) => return None,
                Err(x) => Some(Err(x)),
            }
        }));
    }
}
