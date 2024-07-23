#![allow(dead_code)]

use bytes::{BufMut, Bytes, BytesMut};
use std::{path::PathBuf, str::FromStr};

use crate::errors::HttpError;

/// Serialize data into byte array represented as bytes::Bytes
pub trait Serialize {
    fn serialize(&self) -> Bytes;
}

#[derive(Debug)]
pub enum HttpProtocol {
    Http1_1,
    Http1_0,
}

impl Serialize for HttpProtocol {
    fn serialize(&self) -> Bytes {
        match self {
            HttpProtocol::Http1_1 => Bytes::from(&b"HTTP/1.1"[..]),
            HttpProtocol::Http1_0 => Bytes::from(&b"HTTP/1.0"[..]),
        }
    }
}

impl FromStr for HttpProtocol {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(HttpProtocol::Http1_1),
            "HTTP/1.0" => Ok(HttpProtocol::Http1_0),
            _ => Err(HttpError::ParseProtocolError),
        }
    }
}


#[derive(Debug)]
pub enum HttpStatusCode {
    Ok,
    NotFound,
}

impl Serialize for HttpStatusCode {
    fn serialize(&self) -> Bytes {
        match self {
            HttpStatusCode::Ok => Bytes::from(&b"200 OK"[..]),
            HttpStatusCode::NotFound => Bytes::from(&b"404 Not Found"[..]),
        }
    }
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
}

impl FromStr for HttpMethod {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "PUT" => Ok(HttpMethod::Put),
            "POST" => Ok(HttpMethod::Post),
            _ => Err(HttpError::ParseMethodError),
        }
    }
}

#[derive(Debug)]
pub enum HttpResponseHeader {
    ContentType(Bytes),
    ContentLength(Bytes),
}

impl Serialize for HttpResponseHeader {
    fn serialize(&self) -> Bytes {
        match self {
            HttpResponseHeader::ContentType(str) => {
                let mut header = BytesMut::from(&b"Content-Type: "[..]);
                header.put(&str[..]);
                header.freeze()
            }
            HttpResponseHeader::ContentLength(n) => {
                let mut header = BytesMut::from(&b"Content-Length: "[..]);
                header.put(&n[..]);
                header.freeze()
            }
        }
    }
}

pub struct HttpResponse<'a> {
    protocol: HttpProtocol,
    status: HttpStatusCode,
    headers: Vec<HttpResponseHeader>,
    body: &'a str,
}

impl<'a> HttpResponse<'a> {
    pub fn new() -> Self {
        HttpResponse::default()
    }

    pub fn status(&mut self, status: HttpStatusCode) {
        self.status = status;
    }

    pub fn header(&mut self, header: HttpResponseHeader) {
        self.headers.push(header);
    }

    pub fn headers(&mut self, headers: &mut Vec<HttpResponseHeader>) {
        self.headers.append(headers);
    }

    pub fn body(&mut self, body: &'a str) {
        self.body = body;
    }
}

impl<'a> Default for HttpResponse<'a> {
    fn default() -> Self {
        let protocol = HttpProtocol::Http1_1;
        let status = HttpStatusCode::Ok;
        let headers = Vec::new();
        let body = "";
        Self {
            protocol,
            status,
            headers,
            body,
        }
    }
}

impl<'a> Serialize for HttpResponse<'a> {
    fn serialize(&self) -> Bytes {
        let mut response = BytesMut::new();
        response.put(self.protocol.serialize());
        response.put(&b" "[..]);
        response.put(self.status.serialize());
        response.put(&b"\r\n"[..]);
        for header in &self.headers {
            response.put(header.serialize());
            response.put(&b"\r\n"[..]);
        }
        response.put(&b"\r\n"[..]);
        response.put(self.body.as_bytes());
        response.freeze()
    }
}

/// Structured representation of an HTTP request for more ergonomic handling
pub struct HttpRequest<'a> {
    pub method: HttpMethod,
    pub path: PathBuf,
    pub protocol: HttpProtocol,
    pub headers: Vec<&'a str>,
}

impl<'a> HttpRequest<'a> {
    /// Parse byte array into HttpRequest.
    /// 
    /// # Errors
    ///
    /// Returns `HttpError` if request is not properly formatted.
    pub fn parse(bytes: &'a BytesMut) -> Result<Self, HttpError> {
        let request = std::str::from_utf8(bytes.as_ref())?;
        let mut lines = request.lines();
        let mut start_line = lines
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .split_whitespace();

        let method = start_line
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .parse::<HttpMethod>()?;

        let path = std::path::Path::new(start_line.next().ok_or(HttpError::InvalidRequestFormat)?)
            .to_owned();

        let protocol = start_line
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .parse::<HttpProtocol>()?;

        let mut headers = Vec::new();
        let mut curr_line = lines.next().ok_or(HttpError::InvalidRequestFormat)?;
        while !curr_line.is_empty() {
            headers.push(curr_line);
            curr_line = lines.next().ok_or(HttpError::InvalidRequestFormat)?;
        }
        Ok(Self {
            method,
            path,
            protocol,
            headers,
        })
    }
}
