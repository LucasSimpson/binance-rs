use hex::encode as hex_encode;
use errors::*;
use reqwest;
use reqwest::{Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, CONTENT_TYPE};
use std::io::Read;
use ring::hmac;

static API1_HOST: &'static str = "https://www.binance.com";

#[derive(Clone)]
pub struct Client {
    api_key: String,
    secret_key: String,
}

impl Client {
    pub fn new(api_key: Option<String>, secret_key: Option<String>) -> Self {
        Client {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
        }
    }

    pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<(String)> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::Client::new();
        let response = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn post_signed(&self, endpoint: &str, request: &str) -> Result<(String)> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn delete_signed(&self, endpoint: &str, request: &str) -> Result<(String)> {
        let url = self.sign_request(endpoint, request);
        let client = reqwest::Client::new();
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;

        self.handler(response)
    }

    pub fn get(&self, endpoint: &str, request: &str) -> Result<(String)> {
        let mut url: String = format!("{}{}", API1_HOST, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }

        let response = reqwest::get(url.as_str())?;

        self.handler(response)
    }

    pub fn post(&self, endpoint: &str) -> Result<(String)> {
        let url: String = format!("{}{}", API1_HOST, endpoint);

        let client = reqwest::Client::new();
        let response = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()?;

        self.handler(response)
    }

    pub fn put(&self, endpoint: &str, listen_key: &str) -> Result<(String)> {
        let url: String = format!("{}{}", API1_HOST, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::Client::new();
        let response = client
            .put(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()?;

        self.handler(response)
    }

    pub fn delete(&self, endpoint: &str, listen_key: &str) -> Result<(String)> {
        let url: String = format!("{}{}", API1_HOST, endpoint);
        let data: String = format!("listenKey={}", listen_key);

        let client = reqwest::Client::new();
        let response = client
            .delete(url.as_str())
            .headers(self.build_headers(false)?)
            .body(data)
            .send()?;

        self.handler(response)
    }

    // Request must be signed
    fn sign_request(&self, endpoint: &str, request: &str) -> String {
        let signed_key = hmac::Key::new(hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = hex_encode(hmac::sign(&signed_key, request.as_bytes()).as_ref());

        let request_body: String = format!("{}&signature={}", request, signature);
        let url: String = format!("{}{}?{}", API1_HOST, endpoint, request_body);

        url
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap> {
        let mut custon_headers = HeaderMap::new();

        custon_headers.insert(USER_AGENT, HeaderValue::from_static("binance-rs"));
        if content_type {
            custon_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        }
        custon_headers.insert(HeaderName::from_static("x-mbx-apikey"), HeaderValue::from_str(self.api_key.as_str())?);

        Ok(custon_headers)
    }

    fn handler(&self, mut response: Response) -> Result<(String)> {
        match response.status() {
            StatusCode::OK => {
                let mut body = String::new();
                response.read_to_string(&mut body)?;
                Ok(body)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let error_json: BinanceContentError = response.json()?;

                Err(ErrorKind::BinanceError(error_json.code, error_json.msg, response).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BinanceContentError {
    code: i16,
    msg: String
}
