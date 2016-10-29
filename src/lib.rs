#![cfg_attr(test, deny(warnings))]

extern crate url;
extern crate curl;

extern crate serde;
extern crate serde_json;

#[macro_use] extern crate log;

use url::Url;
use std::collections::HashMap;

use std::str;
use std::io::Read;

use curl::easy::Easy;

/// Configuration of an oauth2 application.
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
    pub auth_url: Url,
    pub token_url: Url,
    pub redirect_url: String,
}

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

impl Config {
    pub fn new(id: &str, secret: &str, auth_url: &str,
               token_url: &str) -> Config {
        Config {
            client_id: id.to_string(),
            client_secret: secret.to_string(),
            scopes: Vec::new(),
            auth_url: Url::parse(auth_url).unwrap(),
            token_url: Url::parse(token_url).unwrap(),
            redirect_url: String::new(),
        }
    }

    pub fn authorize_url(&self, state: String) -> Url {
        let scopes = self.scopes.join(",");
        let mut pairs = vec![
            ("client_id", &self.client_id),
            ("state", &state),
            ("scope", &scopes),
        ];
        if self.redirect_url.len() > 0 {
            pairs.push(("redirect_uri", &self.redirect_url));
        }
        let mut url = self.auth_url.clone();
        url.query_pairs_mut().extend_pairs(
            pairs.iter().map(|&(k, v)| { (k, &v[..]) })
        );
        return url;
    }

    pub fn exchange(&self, code: String) -> Result<Token, String> {
        let mut form = HashMap::new();
        form.insert("client_id", self.client_id.clone());
        form.insert("client_secret", self.client_secret.clone());
        form.insert("code", code);
        form.insert("grant_type", "authorization_code".to_string());

        if self.redirect_url.len() > 0 {
            form.insert("redirect_uri", self.redirect_url.clone());
        }

        let form = url::form_urlencoded::Serializer::new(String::new()).extend_pairs(
            form.iter().map(|(k, v)| { (&k[..], &v[..]) })
        ).finish();

        let form = form.into_bytes();
        let mut form = &form[..];

        let mut easy = Easy::new();

        easy.url(&self.token_url.to_string()[..]).unwrap();
        easy.post(true).unwrap();
        easy.post_field_size(form.len() as u64).unwrap();

        let mut data = Vec::new();
        {
            let mut transfer = easy.transfer();

            transfer.read_function(|buf| {
                Ok(form.read(buf).unwrap_or(0))
            }).unwrap();

            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();

            transfer.perform().unwrap();
        }

        let code = easy.response_code().unwrap();

        if code != 200 {
            return Err(format!("expected `200`, found `{}`", code))
        }

        let token: Token = serde_json::from_str(str::from_utf8(&data).unwrap()).unwrap();

        if token.access_token.len() != 0 {
            Ok(token)
        } else if token.error.len() > 0 {
            Err(format!("error `{}`: {}, see {}", token.error, token.error_desc, token.error_uri))
        } else {
            Err(format!("couldn't find access_token in the response"))
        }
    }
}
