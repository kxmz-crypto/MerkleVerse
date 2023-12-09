rm -rf ./config/cluster_res
mkdir ./config/cluster_res
cargo run -- gen-peers --src ./config/cluster.toml --to ./config/cluster_res