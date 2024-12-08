use oauth2::{
    basic::BasicClient, reqwest::http_client, AuthUrl, AuthorizationCode, ClientId, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::mpsc,
    time::Duration,
};


pub const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &'static str = "https://www.googleapis.com/oauth2/v4/token";
pub const INFO_URL: &'static str = "https://www.googleapis.com/oauth2/v1/userinfo";
pub const REDIRECT_URI: &'static str = "urn:ietf:wg:oauth:2.0:oob";
pub const USER_AGENT: &'static str = "rust-oauth-test/0.1";

