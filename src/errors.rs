use std;
use reqwest;
use url;
use serde_json;
use tungstenite;

error_chain! {
    errors {
        BinanceError(code: i16, msg: String, response: reqwest::Response)
     }

    foreign_links {
        ReqError(reqwest::Error);
        InvalidHeaderError(reqwest::header::InvalidHeaderValue);
        IoError(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        UrlParserError(url::ParseError);
        Json(serde_json::Error);
        Tungstenite(tungstenite::Error);
        TimestampError(std::time::SystemTimeError);
    }
}
