#![allow(dead_code)]

use bytes::{BufMut, Bytes, BytesMut};
use std::{path::PathBuf, str::FromStr};

use crate::errors::HttpError;

#[derive(Debug)]
pub enum Protocol {
    Http1_1,
    Http1_0,
}

impl FromStr for Protocol {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Protocol::Http1_1),
            "HTTP/1.0" => Ok(Protocol::Http1_0),
            _ => Err(HttpError::ParseProtocolError),
        }
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok,
    NotFound,
    Created,
}

#[derive(Debug)]
pub enum Method {
    Get,
    Put,
    Post,
}

impl FromStr for Method {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "PUT" => Ok(Method::Put),
            "POST" => Ok(Method::Post),
            _ => Err(HttpError::ParseMethodError),
        }
    }
}

#[derive(Debug)]
pub enum Header<'a> {
    ContentType(&'a str),
    ContentLength(usize),
}

#[derive(Debug)]
pub struct Response<'a> {
    protocol: Protocol,
    status: StatusCode,
    headers: Vec<Header<'a>>,
    body: Option<&'a [u8]>,
}

impl<'a> Response<'a> {
    pub fn new() -> Self {
        Response::default()
    }

    pub fn status(&mut self, status: StatusCode) {
        self.status = status;
    }

    pub fn header(&mut self, header: Header<'a>) {
        self.headers.push(header);
    }

    pub fn body(&mut self, body: &'a [u8]) {
        self.body = Some(body);
    }

    pub fn content_type(&mut self, content_type: &'a str) {
        self.headers.push(Header::ContentType(content_type));
    }

    pub fn content_len(&mut self, len: usize) {
        self.headers.push(Header::ContentLength(len));
    }

    pub fn build(&self) -> Bytes {
        let protocol = match self.protocol {
            Protocol::Http1_1 => "HTTP/1.1",
            Protocol::Http1_0 => "HTTP/1.0",
        };

        let status = match self.status {
            StatusCode::Ok => "200 OK",
            StatusCode::NotFound => "404 Not Found",
            StatusCode::Created => "201 Created",
        };

        let mut metadata = format!("{} {}\r\n", protocol, status);
        for header in &self.headers {
            let header = match header {
                Header::ContentType(content) => format!("Content-Type: {}", content),
                Header::ContentLength(n) => format!("Content-Length: {}", n),
            };
            metadata.push_str(header.as_str());
            metadata.push_str("\r\n");
        }
        metadata.push_str("\r\n");
        let mut response = BytesMut::from(metadata.as_bytes());
        if let Some(body) = self.body {
            response.put(body);
        }
        response.freeze()
    }

    /// Build simple OK response with no headers or body
    pub fn ok() -> Bytes {
        let protocol = Protocol::Http1_1;
        let status = StatusCode::Ok;
        let headers = Vec::new();
        let body = None;
        Self {
            protocol,
            status,
            headers,
            body,
        }.build()
    }

    /// Build a simple 404 Not Found error response
    pub fn not_found() -> Bytes {
        let protocol = Protocol::Http1_1;
        let status = StatusCode::NotFound;
        let headers = Vec::new();
        let body = None;
        Self {
            protocol,
            status,
            headers,
            body,
        }.build()
    }
}

impl<'a> Default for Response<'a> {
    fn default() -> Self {
        let protocol = Protocol::Http1_1;
        let status = StatusCode::Ok;
        let headers = Vec::new();
        let body = None;
        Self {
            protocol,
            status,
            headers,
            body,
        }
    }
}

/// Structured representation of an HTTP request for more ergonomic handling
pub struct Request<'a> {
    pub method: Method,
    pub path: PathBuf,
    pub protocol: Protocol,
    pub headers: Vec<&'a str>,
    pub body: Option<Bytes>
}

impl<'a> Request<'a> {
    /// Parse byte array into Request.
    ///
    /// # Errors
    ///
    /// Returns `HttpError` if request is not properly formatted.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, HttpError> {
        let request = std::str::from_utf8(bytes)?;
        let mut lines = request.lines();
        let mut start_line = lines
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .split_whitespace();

        let method = start_line
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .parse::<Method>()?;

        let path = std::path::Path::new(start_line.next().ok_or(HttpError::InvalidRequestFormat)?)
            .to_owned();

        let protocol = start_line
            .next()
            .ok_or(HttpError::InvalidRequestFormat)?
            .parse::<Protocol>()?;

        let mut headers = Vec::new();
        let mut curr_line = lines.next().ok_or(HttpError::InvalidRequestFormat)?;
        while !curr_line.is_empty() {
            headers.push(curr_line);
            curr_line = lines.next().ok_or(HttpError::InvalidRequestFormat)?;
        }
        let body = Some(Bytes::from(lines.collect::<String>()));
        Ok(Self {
            method,
            path,
            protocol,
            headers,
            body,
        })
    }
}
