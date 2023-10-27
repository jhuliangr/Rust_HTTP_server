use super::method::{Method, MethodError}; 
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{ Debug, Display, Formatter, Result as FmtResult};
use std::str::{self, Utf8Error};
use super::QueryString;

#[derive(Debug)]
pub struct Request<'buf> {
    path: &'buf str,
    query_string: Option<QueryString<'buf>>,
    method: Method,
    headers: &'buf str,
    body: &'buf str
}

impl<'buf> Request<'buf>{
    pub fn path(&self) -> &str{
        &self.path
    }
    pub fn method(&self) ->&Method{
        &self.method
    }
    pub fn query_string(&self) -> Option<&QueryString>{
        self.query_string.as_ref()
    }
    pub fn headers(&self) -> &str {
        &self.headers
    }  
    pub fn body(&self) -> &str {
        &self.body
    }  
}

impl<'buf> TryFrom<&'buf [u8]> for Request<'buf>{
    type Error = ParseError;

    // GET /test?name=nombre&sort=1 HTTP/1.1\r\n...headers...
    fn try_from(buf: &'buf [u8]) -> Result<Request<'buf>, Self::Error>{
        let request = str::from_utf8(buf)?;

        let (method , request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (mut path, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;
        let (protocol, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;

        let (headers, body) = separe_headers_and_body(request).ok_or(ParseError::InvalidRequest)?;


        
        if "HTTP/1.1" != protocol{
            return Err(ParseError::InvalidProtocol)
        }
        let method: Method = method.parse()?;

        let mut query_string = None;
        if let Some(i) = path.find('?'){
            query_string = Some(QueryString::from(&path[i+1..]));
            path = &path[..i];
        }

        Ok(Self { path , query_string, method, headers, body })
    }
}
fn get_next_word(request: &str) -> Option<(&str, &str)> {
    for (i, c) in request.chars().enumerate(){
         if c == ' ' || c == '\r' {
            return Some((&request[..i], &request[i+1..]));
         }
    }
    None

}
fn separe_headers_and_body(request: &str) -> Option<(&str, &str)> {
    let mut start = 0;
    let mut end = 0;
    for (i, c) in request.chars().enumerate(){

        if c == '{' && start == 0 {
        // return Some((&request[..i-1], &request[i..]));
            start = i;
        }
        if c =='}' {
            end = i;
        }
    }
    if start == 0 {
        return None
    }
    Some((&request[..start], &request[start..end+1]))

}

pub enum ParseError {
    InvalidRequest,
    InvalidEncoding,
    InvalidProtocol,
    InvalidMethod
}
impl From<Utf8Error> for ParseError{
    fn from(_: Utf8Error) -> Self{
        ParseError::InvalidEncoding
    }
}
impl From<MethodError> for ParseError{
    fn from(_: MethodError) -> Self{
        ParseError::InvalidMethod
    }
}

impl ParseError {
    fn message(&self) -> &str{
        match self {
            Self::InvalidRequest => "Invalid Request", 
            Self::InvalidEncoding => "Invalid Encoding",    
            Self::InvalidProtocol => "Invalid Protocol",    
            Self::InvalidMethod => "Invalid Method",    
        }
    }
}
impl Display for ParseError{
    fn fmt(&self, f:&mut Formatter) -> FmtResult{
        write!(f, "{}", self.message())
    }
}
impl Debug for ParseError{
    fn fmt(&self, f:&mut Formatter) -> FmtResult{
        write!(f, "{}", self.message())
    }
}

impl Error for ParseError {}
