#![warn(bad_style)]
// #![warn(missing_docs)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]

#[macro_use]
extern crate log;
extern crate oauth_client as oauth;
extern crate rustc_serialize as rustc_serialize;

use std::borrow::Cow;
use std::collections::HashMap;
use oauth::Token;
use rustc_serialize::Decodable;
use rustc_serialize::json::{self, Json};

pub use error::Error;

mod error;

mod api_twitter_oauth {
    pub const REQUEST_TOKEN: &'static str = "https://api.twitter.com/oauth/request_token";
    pub const AUTHORIZE: &'static str = "https://api.twitter.com/oauth/authorize";
    pub const ACCESS_TOKEN: &'static str = "https://api.twitter.com/oauth/access_token";
}

mod api_twitter_soft {
    pub const UPDATE_STATUS: &'static str = "https://api.twitter.com/1.1/statuses/update.json";
    pub const DIRECT_MESSAGE: &'static str = "https://api.twitter.com/1.1/direct_messages/new.json";
    pub const DIRECT_MESSAGES: &'static str = "https://api.twitter.com/1.1/direct_messages.json";
    pub const DESTROY_STATUS: &'static str = "https://api.twitter.com/1.1/statuses/destroy/";
    pub const HOME_TIMELINE: &'static str = "https://api.twitter.com/1.1/statuses/home_timeline.json";
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct TwitterUser {
    pub screen_name: String,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Tweet {
    pub created_at: String,
    pub full_text: String,
    pub id: u64,
    pub user: TwitterUser,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct DirectMessage {
    pub created_at: String,
    pub text: String,
    pub id: u64,
    pub sender_screen_name: String,
}

impl Tweet {
    pub fn parse_timeline(json_string: String) -> Result<Vec<Tweet>, Error> {
        let conf = try!(Json::from_str(&json_string));
        let d = try!(Decodable::decode(&mut json::Decoder::new(conf)));
        Ok(d)
    }
}

impl DirectMessage {
    pub fn parse_timeline(json_string: String) -> Result<Vec<DirectMessage>, Error> {
        let conf = try!(Json::from_str(&json_string));
        let d = try!(Decodable::decode(&mut json::Decoder::new(conf)));
        Ok(d)
    }
}


fn split_query<'a>(query: &'a str) -> HashMap<Cow<'a, str>, Cow<'a, str>> {
    let mut param = HashMap::new();
    for q in query.split('&') {
        let mut s = q.splitn(2, '=');
        let k = s.next().unwrap();
        let v = s.next().unwrap();
        let _ = param.insert(k.into(), v.into());
    }
    param
}

pub fn get_request_token(consumer: &Token) -> Result<Token<'static>, Error> {
    let bytes = try!(oauth::get(api_twitter_oauth::REQUEST_TOKEN, consumer, None, None));
    let resp = try!(String::from_utf8(bytes));
    let param = split_query(&resp);
    let token = Token::new(param.get("oauth_token").unwrap().to_string(),
                           param.get("oauth_token_secret").unwrap().to_string());
    Ok(token)
}

pub fn get_authorize_url(request: &Token) -> String {
    format!("{}?oauth_token={}",
            api_twitter_oauth::AUTHORIZE,
            request.key)
}

pub fn get_access_token(consumer: &Token,
                        request: &Token,
                        pin: &str)
                        -> Result<Token<'static>, Error> {
    let mut param = HashMap::new();
    let _ = param.insert("oauth_verifier".into(), pin.into());
    let bytes = try!(oauth::get(api_twitter_oauth::ACCESS_TOKEN,
                                consumer,
                                Some(request),
                                Some(&param)));
    let resp = try!(String::from_utf8(bytes));
    let param = split_query(&resp);
    let token = Token::new(param.get("oauth_token").unwrap().to_string(),
                           param.get("oauth_token_secret").unwrap().to_string());
    Ok(token)
}

/// function to update the status
/// This function takes as arguments the consumer key, the access key, and the status (obviously)
pub fn update_status(consumer: &Token, access: &Token, status: &str) -> Result<(), Error> {
    let mut param = HashMap::new();
    let _ = param.insert("status".into(), status.into());
    let _ = try!(oauth::post(api_twitter_soft::UPDATE_STATUS,
                             consumer,
                             Some(access),
                             Some(&param)));
    Ok(())
}

pub fn direct_message(consumer: &Token, access: &Token, text: &str, screen_name: &str) -> Result<(), Error> {
    let mut param = HashMap::new();
    let _ = param.insert("text".into(), text.into());
    let _ = param.insert("screen_name".into(), screen_name.into());
    let _ = try!(oauth::post(api_twitter_soft::DIRECT_MESSAGE,
                             consumer,
                             Some(access),
                             Some(&param)));
    Ok(())
}

pub fn destroy_status(consumer: &Token, access: &Token, id: &u64) -> Result<(), Error> {
    let param = HashMap::new();
    let _ = try!(oauth::post(format!("{}{}.json", api_twitter_soft::DESTROY_STATUS, id).as_str(),
                             consumer,
                             Some(access),
                             Some(&param)));
    Ok(())
}

pub fn get_direct_messages(consumer: &Token, access: &Token) -> Result<Vec<DirectMessage>, Error> {
    let bytes = try!(oauth::get(api_twitter_soft::DIRECT_MESSAGES,
                                consumer,
                                Some(access),
                                None));
    let last_tweets_json = try!(String::from_utf8(bytes));
    let ts = try!(DirectMessage::parse_timeline(last_tweets_json));
    Ok(ts)
}

pub fn get_last_tweets(consumer: &Token, access: &Token) -> Result<Vec<Tweet>, Error> {
    let mut param = HashMap::new();
    let _ = param.insert("tweet_mode".into(), "extended".into());

    let bytes = try!(oauth::get(api_twitter_soft::HOME_TIMELINE,
                                consumer,
                                Some(access),
                                Some(&param)));
    let last_tweets_json = try!(String::from_utf8(bytes));
    let ts = try!(Tweet::parse_timeline(last_tweets_json));
    Ok(ts)
}
