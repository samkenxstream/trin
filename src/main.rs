use clap::{App, Arg};
use tokio::runtime::Builder;
use trin::{discovery::Enr, Service};

mod cli;

fn main() {
    let matches = App::new("trin")
        .about("Run a trin node")
        .settings(&[clap::AppSettings::ColoredHelp])
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .value_name("PORT")
                .help("The UDP port to listen on.")
                .default_value("9000")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("boot-nodes")
                .long("boot-nodes")
                .allow_hyphen_values(true)
                .value_name("ENR-LIST")
                .help("One or more comma-delimited base64-encoded ENR's or multiaddr strings of peers to initially add to the local routing table")
                .takes_value(true),
        ).get_matches();

    env_logger::init();
    let runtime = Builder::new_multi_thread().enable_all().build().unwrap();

    let boot_nodes = {
        let mut boot_nodes = Vec::new();

        if let Some(nodes) = matches.value_of("boot-nodes") {
            boot_nodes.extend_from_slice(
                &nodes
                    .split(',')
                    .map(|enr| enr.parse().map_err(|_| format!("Invalid ENR: {}", enr)))
                    .collect::<Result<Vec<Enr>, _>>()
                    .unwrap(),
            );
        }

        boot_nodes
    };

    let port = matches.value_of("port").unwrap().parse().unwrap();
    runtime.block_on(async move {
        let mut service = Service::new(port, boot_nodes).await;
        service.find_peers_loop().await;
    });
}
