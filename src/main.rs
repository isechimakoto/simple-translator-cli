use std::{env, io::{Read, self, Write, Error, ErrorKind}, fs::File};

use clap::Parser;
use dotenv::dotenv;
use reqwest::header::HeaderValue;
use serde::Deserialize;

#[derive(Parser, Debug)]
struct CliArgs {
    #[clap(short = 'f', long = "from-lang")]
    from_lang: String,
    #[clap(short = 't', long = "to-lang")]
    to_lang: String,
    from_path: std::path::PathBuf,
    to_path: std::path::PathBuf,
}

#[derive(Deserialize, Debug)]
struct Sentence {
    trans: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Response {
    sentences: Vec<Sentence>,
}

#[tokio::main]
async fn main() -> io::Result<()>{
    let args = CliArgs::parse();

    let mut f = File::open(args.from_path.as_path()).unwrap_or_else(|error| {
        panic!("Error caused by from_path option: {:?}", error.to_string())
    });
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;

    dotenv().ok();
    let translate_api_id = env::var("TRANSLATE_API_ID").unwrap();

    let client = reqwest::Client::new();
    let params = [
        ("sl", args.from_lang),
        ("tl", args.to_lang),
        ("q", buffer),
    ];

    let url = format!("{}{}", "https://translate.google.com/translate_a/single?client=at&dt=t&dt=ld&dt=qca&dt=rm&dt=bd&dj=1&hl=ja&ie=UTF-8&oe=UTF-8&inputm=2&otf=2&iid=", translate_api_id.as_str());
    let res = client.post(url)
        .header(reqwest::header::CONTENT_TYPE,  HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"))
        .header(reqwest::header::USER_AGENT, HeaderValue::from_static("AndroidTranslate/5.3.0.RC02.130475354-53000263 5.1 phone TRANSLATE_OPM5_TEST_1"))
        .form(&params)
        .send()
        .await
        .unwrap();

    let mut tf =  File::create(args.to_path)?;
    let translated = match res.json::<Response>().await {
        Ok(translated) => translated,
        Err(e) => {
            eprintln!("Fail to deserialize json: err = {:?}", e);
            Response {
                sentences: vec![],
            }
        }
    };

    match &translated.sentences[0].trans {
        Some(trans) => {
            tf.write_fmt(format_args!("{}", trans)).unwrap();
            Ok(())
        },
        _ => Err(Error::new(ErrorKind::Other, "Not found translated text")),
    }
}
