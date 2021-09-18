mod attributes;

use crate::attributes::ParsePathToAttributes;
use rouille;
use rouille::router;
use std::fs::File;
use std::io::BufReader;
use std::option::Option::Some;
use std::path::PathBuf;
use structopt::StructOpt;
use std::collections::HashMap;

/// Mock Google Compute Engine Metadata Server
///
/// accepts metadata in [{"key": "<key>", "value": "<value>"}, ...] format
#[derive(Debug, StructOpt)]
struct Opt {
    /// Metadata file
    #[structopt(parse(from_os_str))]
    metadata_file: PathBuf,

    /// Server listen address
    #[structopt(short, long, default_value = "127.0.0.1:80")]
    listen_address: String,

    /// json/yaml pointer (https://tools.ietf.org/html/rfc6901) to the metadata array
    #[structopt(short("p"), long, default_value = "")]
    metadata_path: String,

    /// is json
    #[structopt(short, long)]
    json: bool,
}

fn main() {
    let opt = Opt::from_args();

    let path: PathBuf = opt.metadata_file.clone();
    let file = File::open(path).expect("Unable to open metadata file");

    let metadata = if opt.json {
        let json: serde_json::Value =
            serde_json::from_reader(BufReader::new(file)).expect("Metadata file not valid");
        json.parse(&opt.metadata_path)
    } else {
        let yaml: serde_yaml::Value =
            serde_yaml::from_reader(BufReader::new(file)).expect("Metadata file not valid");
        yaml.parse(&opt.metadata_path)
    };

    let attrs: HashMap<_, _> = metadata.into_iter().map(|attr| {
        (attr.key, attr.value)
    }).collect();

    println!("Listening on {}", opt.listen_address);
    rouille::start_server(opt.listen_address, move |r| {
        if r.header("Host")
            .map_or(false, |h| h == "metadata.google.internal")
            && r.header("X-Google-Metadata-Request")
                .map_or(false, |h| h == "True")
        {
            router!(r,
                (GET) (/computeMetadata/v1/instance/attributes/{attribute: String}) => {
                    if let Some(attr) = attrs.get(&attribute) {
                        rouille::Response::text(attr)
                    } else {
                        rouille::Response::empty_404()
                    }
                },
                _ => rouille::Response::empty_404(),
            )
        } else {
            rouille::Response::empty_404()
        }
    });
}
