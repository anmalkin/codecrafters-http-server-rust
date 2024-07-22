use std::path::PathBuf;

use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub enum HttpProtocol {
    Http1_1,
    Http1_0,
}

impl HttpProtocol {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            HttpProtocol::Http1_1 => Bytes::from("HTTP/1.1 "),
            HttpProtocol::Http1_0 => Bytes::from("HTTP/1.0 "),
        }
    }
}

pub enum HttpStatusCode {
    _200_,
    _400_,
}

impl HttpStatusCode {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            HttpStatusCode::_200_ => Bytes::from("200 OK"),
            HttpStatusCode::_400_ => Bytes::from("404 Not Found"),
        }
    }
}

#[derive(Debug)]
pub enum HttpMethod {
    GET,
    PUT,
    POST,
}

#[derive(Debug)]
pub enum HttpResponseHeader {
    ContentType(Bytes),
    ContentLength(Bytes),
}

impl HttpResponseHeader {
    pub fn as_bytes(&self) -> Bytes {
        match self {
            HttpResponseHeader::ContentType(str) => {
                let mut header = BytesMut::from("Content-Type: ");
                header.put(&str[..]);
                header.freeze()
            }
            HttpResponseHeader::ContentLength(n) => {
                let mut header = BytesMut::from("Content-Length: ");
                header.put(&n[..]);
                header.freeze()
            }
        }
    }
}

#[derive(Debug)]
pub enum HttpRequestHeader {
    Host(Bytes),
    UserAgent(Bytes),
}

struct HttpResponse {
    protocol: HttpProtocol,
    status: HttpStatusCode,
    headers: Vec<HttpResponseHeader>,
    body: BytesMut,
}

impl HttpResponse {
    pub fn new(
        protocol: HttpProtocol,
        status: HttpStatusCode,
        headers: Vec<HttpResponseHeader>,
        body: BytesMut,
    ) -> Self {
        Self {
            protocol,
            status,
            headers,
            body,
        }
    }

    pub fn as_bytes(&self) -> Bytes {
        let mut response = BytesMut::new();
        response.put(self.protocol.as_bytes());
        response.put(self.status.as_bytes());
        response.put("\r\n".as_bytes());
        for header in &self.headers {
            response.put(header.as_bytes());
            response.put("\r\n".as_bytes());
        }
        response.put("\r\n".as_bytes());
        response.put(self.body.clone());
        response.freeze()
    }
}

impl Default for HttpResponse {
    fn default() -> Self {
        let protocol = HttpProtocol::Http1_1;
        let status = HttpStatusCode::_200_;
        let content_len = Bytes::from("0");
        let headers = vec![HttpResponseHeader::ContentLength(content_len)];
        let body = BytesMut::new();
        Self {
            protocol,
            status,
            headers,
            body,
        }
    }
}

pub struct HttpRequest {
    method: HttpMethod,
    path: PathBuf,
    protocol: HttpProtocol,
    headers: Vec<HttpRequestHeader>,
}
