use anyhow::{bail, Result};
use embedded_svc::http::{client::Client, Method};
use esp_idf_svc::http::client::{Configuration, EspHttpConnection, FollowRedirectsPolicy};
use std::fmt;
use urlencoding::encode;

macro_rules! fatal {
    ($($arg:tt)*) => {{
        let formatted_message = format!($($arg)*);
        log::error!("{}", formatted_message);
        bail!(formatted_message);
    }};
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BuptAccount {
    username: String,
    password: String,
}

impl fmt::Debug for BuptAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let password_length = self.password.len();
        let hidden_password = "*".repeat(password_length);
        f.debug_struct("BuptAccount")
            .field("username", &self.username)
            .field("password", &hidden_password)
            .finish()
    }
}

const CHECK_URL: &str = "http://connect.rom.miui.com/generate_204?cmd=redirect&arubalp=12345";

enum BuptNetStatus {
    Authenticated,
    NotAuthenticated(Option<String>),
}

fn check(url: impl AsRef<str>) -> Result<BuptNetStatus> {
    log::debug!("checking bupt network status with url: {}", url.as_ref());
    let connection = EspHttpConnection::new(&Configuration{
        follow_redirects_policy: FollowRedirectsPolicy::FollowNone,
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);
    let request = client.request(Method::Get, url.as_ref(), &[])?;
    let response = request.submit()?;
    log::debug!("response status: {}", response.status());
    match response.status() {
        // Logged in, not redirected
        204 => Ok(BuptNetStatus::Authenticated),
        // Redirect to login page
        302 => {
            let location = response
                .header("Location")
                .ok_or_else(|| anyhow::anyhow!("no Location header found in response"))?;
            log::info!("redirected to: {}", location);
            check(location)
        }
        // Redirected to login page
        200 => Ok(BuptNetStatus::NotAuthenticated(
            response
                .header("Set-Cookie")
                .and_then(|cookie| cookie.split(';').next().map(|cookie| cookie.to_string())),
        )),
        _ => fatal!("unexpected status code: {}", response.status()),
    }
}

fn auth(account: BuptAccount, cookie: String) -> Result<()> {
    let connection = EspHttpConnection::new(&Configuration::default())?;
    let mut client = Client::wrap(connection);
    let headers = [
        ("Content-Type", "application/x-www-form-urlencoded"),
        ("Cookie", &cookie),
    ];
    let mut request = client.request(Method::Post, "http://10.3.8.216/login", &headers)?;
    request.write(
        format!(
            "user={}&pass={}",
            encode(&account.username),
            encode(&account.password)
        )
        .as_bytes(),
    )?;
    let mut response = request.submit()?;
    log::debug!("response status: {}", response.status());
    match response.status() {
        302 => {
            let location = response
                .header("Location")
                .ok_or_else(|| anyhow::anyhow!("no Location header found in response"))?;
            fatal!("unexpected redirect: {}", location)
        }
        200 => match check(CHECK_URL)? {
            BuptNetStatus::Authenticated => {
                log::info!("BUPT-portal authenticated successfully");
                Ok(())
            }
            _ => {
                let mut data: Vec<u8> = Vec::new();
                loop {
                    let mut buffer = [0u8; 1024];
                    if let Ok(size) = response.read(&mut buffer) {
                        if size == 0 {
                            break;
                        }
                        data.extend_from_slice(&buffer[..size]);
                    } else {
                        break;
                    }
                }
                let body = String::from_utf8(data)
                    .map_err(|e| anyhow::anyhow!("failed to parse response body: {}", e))?;
                let reason = body.find("<div class=\"ui error message\">").map_or(
                    "Unknown error",
                    |start| {
                        body[start..].find("</div>").map_or("Unknown error", |end| {
                            body[(start + 30)..(start + end)].trim()
                        })
                    },
                );
                fatal!("BUPT-portal authentication failed: {}", reason)
            }
        },
        _ => fatal!("unexpected status code: {}", response.status()),
    }
}

pub fn login(account: BuptAccount) -> Result<()> {
    log::info!("Checking BUPT-portal status...");
    match check(CHECK_URL) {
        Ok(BuptNetStatus::Authenticated) => {
            log::info!("BUPT-portal is already authenticated");
            Ok(())
        }
        Ok(BuptNetStatus::NotAuthenticated(cookie)) => {
            log::info!(
                "BUPT-portal not authenticated, authenticating with account: {:?}",
                account,
            );
            auth(
                account,
                cookie.map_or_else(
                    || {
                        log::warn!("No cookie found in response, may not be able to authenticate");
                        String::new()
                    },
                    |cookie| {
                        log::info!("Cookie: {}", &cookie);
                        cookie
                    },
                ),
            )?;
            Ok(())
        }
        Err(e) => fatal!("BUPT-portal status check failed: {}", e),
    }
}

pub fn get(url: impl AsRef<str>) -> Result<String> {
    let connection = EspHttpConnection::new(&Configuration::default())?;
    let mut client = Client::wrap(connection);
    let request = client.request(Method::Get, url.as_ref(), &[])?;
    let mut response = request.submit()?;
    let mut body: Vec<u8> = Vec::new();
    loop {
        let mut buffer = [0u8; 1024];
        if let Ok(size) = response.read(&mut buffer) {
            if size == 0 {
                break;
            }
            body.extend_from_slice(&buffer[..size]);
        } else {
            break;
        }
    }
    Ok(String::from_utf8(body).unwrap_or_else(|_| "".to_string()))
}
