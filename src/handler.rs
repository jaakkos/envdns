use crate::resolver::{resolve, ResolverError};
use crate::Options;
use std::net::IpAddr;
use std::{borrow::Borrow, str::FromStr, sync::Arc};
use tracing::*;

use trust_dns_server::{
    authority::MessageResponseBuilder,
    client::rr::{LowerName, Name, RData, Record},
    proto::op::{Header, MessageType, OpCode, ResponseCode},
    server::{Request, RequestHandler, ResponseHandler, ResponseInfo},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid OpCode {0:}")]
    InvalidOpCode(OpCode),
    #[error("Invalid MessageType {0:}")]
    InvalidMessageType(MessageType),
    #[error("IO error: {0:}")]
    Io(#[from] std::io::Error),
    #[error("Resolver error: {0:}")]
    Resolver(#[from] ResolverError),
}

// DNS Request Handler
#[derive(Clone, Debug)]
pub struct Handler {
    // Domain to serve DNS responses for (requests for other domains are silently ignored).
    pub root_zone: LowerName,
    // Zone name for counter (counter.envdens.local)
    pub counter_zone: LowerName,
    // Zone name for myip (myip.envdens.local)
    pub myip_zone: LowerName,
    // Zone name for hello (hello.envdens.local)
    pub hello_zone: LowerName,
    // Entries to resolve locally
    pub entries: Arc<Vec<(LowerName, IpAddr)>>,
}

impl Handler {
    // Create new handler from command-line options.
    pub fn from_options(options: &Options) -> Self {
        let domain = &options.domain;
        Handler {
            root_zone: LowerName::from(Name::from_str(domain).unwrap()),
            counter_zone: LowerName::from(Name::from_str(&format!("counter.{domain}")).unwrap()),
            myip_zone: LowerName::from(Name::from_str(&format!("myip.{domain}")).unwrap()),
            hello_zone: LowerName::from(Name::from_str(&format!("hello.{domain}")).unwrap()),
            entries: Arc::new(
                options
                    .entries
                    .iter()
                    .map(|entry| {
                        let parts: Vec<&str> = entry.split(':').collect();
                        let name = LowerName::from_str(parts[0]).unwrap();
                        let ip = IpAddr::from_str(parts[1]).unwrap();
                        (name, ip)
                    })
                    .collect(),
            ),
        }
    }

    /// Checks if a given query name is in the entries
    fn is_query_in_entries(&self, query_name: &LowerName) -> bool {
        // Extracting the string representation from the LowerName
        let name_string = query_name.to_string();

        info!("Query name: {:#?}", name_string);
        self.entries
            .iter()
            .any(|(entry_name, _)| entry_name == query_name)
    }

    // Handle requests for myip.{domain}.
    async fn do_handle_request_myip<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        let rdata = match request.src().ip() {
            IpAddr::V4(ipv4) => RData::A(ipv4),
            IpAddr::V6(ipv6) => RData::AAAA(ipv6),
        };
        let records = vec![Record::from_rdata(request.query().name().into(), 60, rdata)];
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(responder.send_response(response).await?)
    }

    // Handle requests for anything else (NXDOMAIN)
    async fn do_handle_request_default<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        header.set_response_code(ResponseCode::NXDomain);
        let response = builder.build_no_records(header);
        Ok(responder.send_response(response).await?)
    }

    async fn do_handle_request_other<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        let name: Name = request.query().name().into();

        // Use the resolve function
        match resolve(name.clone()).await {
            Ok(records) => {
                let builder = MessageResponseBuilder::from_message_request(request);
                let mut header = Header::response_from_request(request.header());
                header.set_authoritative(false); // since we're just proxying

                let response = builder.build(header, records.iter(), &[], &[], &[]);
                Ok(responder.send_response(response).await?)
            }
            Err(e) => Err(Error::Resolver(e)), // Handle the error accordingly. Here I'm converting the ResolverError into your custom Error type.
        }
    }

    async fn do_handle_request_entry<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, Error> {
        let name: &Name = request.query().name().borrow();
        // convert Name to LowerName
        let name_lower: LowerName = name.into();

        // Since we're sure the entry exists, we can just unwrap it
        let entry = self
            .entries
            .iter()
            .find(|(entry_name, _)| entry_name == &name_lower)
            .unwrap();

        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);

        let rdata = match entry.1 {
            IpAddr::V4(ipv4) => RData::A(ipv4),
            IpAddr::V6(ipv6) => RData::AAAA(ipv6),
        };

        let records = vec![Record::from_rdata(request.query().name().into(), 60, rdata)];
        let response = builder.build(header, records.iter(), &[], &[], &[]);

        Ok(responder.send_response(response).await?)
    }

    // Handle request, returning ResponseInfo if response was successfully sent, or an error.
    async fn do_handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> Result<ResponseInfo, Error> {
        // make sure the request is a query
        if request.op_code() != OpCode::Query {
            return Err(Error::InvalidOpCode(request.op_code()));
        }

        // make sure the message type is a query
        if request.message_type() != MessageType::Query {
            return Err(Error::InvalidMessageType(request.message_type()));
        }

        // print entries
        info!("Entries: {:#?}", self.entries);

        match request.query().name() {
            name if self.is_query_in_entries(name) => {
                self.do_handle_request_entry(request, response).await
            }
            name if self.myip_zone.zone_of(name) => {
                self.do_handle_request_myip(request, response).await
            }
            name if self.root_zone.zone_of(name) => {
                self.do_handle_request_default(request, response).await
            }
            _name => self.do_handle_request_other(request, response).await,
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> ResponseInfo {
        // try to handle request
        match self.do_handle_request(request, response).await {
            Ok(info) => info,
            Err(error) => {
                error!("Error in RequestHandler: {error}");
                let mut header = Header::new();
                header.set_response_code(ResponseCode::ServFail);
                header.into()
            }
        }
    }
}
