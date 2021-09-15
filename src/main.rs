use json::JsonValue;
use clap::{Arg, App};

use std::io::BufWriter;
use std::fs::File;
use std::fmt;
use std::path::Path;


struct Candle{
    complete : bool,
    volume : i32,
    time : String,
    mid : Vec<f32>
}

impl fmt::Display for Candle{

    fn fmt(&self, f: &mut fmt::Formatter<'_> ) -> fmt::Result{

        let mut out_string = String::new();
        let string_vec : Vec<String> = self.mid.iter().map(|value| value.to_string()).collect();
        for item in string_vec{
            out_string.push_str(item.as_str());
            out_string.push(',');
        }

        write!(f, "{},{},{},{}", self.time, self.complete, self.volume, out_string)
    }
}

struct OandaClient{

    client : reqwest::Client,
    async_runtime : tokio::runtime::Runtime,
    key :  String,
    account :  String,

}

impl OandaClient{

    fn new(key : &str, account : &str) -> OandaClient{
        OandaClient{ client : reqwest::Client::new(),
                     async_runtime : tokio::runtime::Runtime::new().unwrap(),
                     key : key.to_string(),
                     account : account.to_string()}

    }

    async fn make_request(&self, url : String, auth_header : String) -> JsonValue{

        let res = self.client.get(url)
                             .header("Content-Type", "application/json")
                             .header("Authorization", auth_header)
                             .send().await.unwrap();

         let body = res.text().await.unwrap();
         json::parse(&body).unwrap()
    }

    fn get_instrument_candles(&self, instrument : &str, count : i32 ,granularity : &str) -> Vec<Candle>{

        let url = format!("https://api-fxpractice.oanda.com/v3/instruments/{}/candles?count={}&price=M&granularity={}", instrument, count, granularity);
        let auth_header = format!("Bearer {}", self.key);
        let mut output = self.async_runtime.block_on(self.make_request(url, auth_header));

        let candles : &JsonValue = &output.remove("candles");
        let candles : Vec::<Candle> = unpack_candles(candles);

        candles
    }
}

fn unpack_candles(candles : &JsonValue) -> Vec<Candle>{

    let mut out_candles : Vec<Candle> = Vec::new();
    for item in candles.members(){
        let mut mid : Vec<f32> = Vec::new();
        for value in item["mid"].entries(){
            mid.push(value.1.as_str().unwrap().parse::<f32>().unwrap())
        }

        out_candles.push(Candle { complete : item["complete"].as_bool().unwrap(),
                                  volume : item["volume"].as_i32().unwrap(),
                                  time : item["time"].as_str().unwrap().to_string(),
                                  mid : mid})
    }

    out_candles
}

fn main(){

    let account = "101-001-10756720-001";
    let key = "e3875962ab63e8d392e507e8f416aecf-c901d043e6f53a4100c3cef1d8b52120";

    let client = OandaClient::new(key, account);

    let instrument = "EUR_USD";
    let count = 1000;
    let granularity = "M5";

    let mut output = client.get_instrument_candles(instrument, count, granularity);

    let mut filename = String::new();

    filename.push_str("test_name_pls_ignore");
    filename.push('_');
    filename.push_str(instrument);
    filename.push('_');
    filename.push_str(granularity);
    filename.push('_');
    filename.push_str(count.to_string().as_str());
    filename.push_str(".csv");

    let mut out_file = File::create(filename);
    let mut out_buffer = BufWriter::new(out_file);


}

/*
let mut outfile = BufWriter::new(File::create("price_test.csv").unwrap());

curl \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer e3875962ab63e8d392e507e8f416aecf-c901d043e6f53a4100c3cef1d8b52120" \
  "https://api-fxtrade.oanda.com/v3/instruments/EUR_USD/candles?count=6&price=M&granularity=S5"

*/
