extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate url;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use url::Url;

pub struct SonosApi {
    core: Core,
    client: Client<hyper::client::HttpConnector>,
    uri: String
}

pub struct SonosRoom {
    api: SonosApi,
    room: String
}

impl<'a> SonosApi {
    pub fn new(uri: String) -> SonosApi {
        let mut core = Core::new().unwrap();
        let client = Client::new(&core.handle());

        SonosApi {core: core, client: client, uri: uri}
    }

    pub fn room(self, room: String) -> SonosRoom {
        SonosRoom {api: self, room: room}
    }
}

impl SonosRoom {
    pub fn say(&mut self, sentence: String){
        let mut uri = self.api.uri.clone();
        uri.push_str(&format!{"/{}/{}", self.room, sentence});
        uri = uri.replace(" ", "%20");
        let work = self.api.client.get(uri.parse().unwrap())
            .map(|res| {
                println!("Response: {}", res.status());
            });
        let parsed_uri : hyper::Uri = uri.parse().unwrap();
        self.api.core.run(work).unwrap_or_else(|err| {
            println!{"An error occured calling the sonos REST api with {}:\n{:?}", parsed_uri, err};
        });
    }
}
