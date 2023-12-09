# Define the number of servers
SRV_LEN=1

# Function to run server commands
run_server() {
    local i=$1
    RUST_BACKTRACE=1 RUST_LOG=debug target/debug/MerkleVerseWrapper server -c config/cluster_res/srv_${i}.toml &
    PORT=$((6000+i)) ../MerkleSquare/demo/mverserver/mverserver &
}

# Start all servers
for ((i=0; i<SRV_LEN; i++)); do
    run_server $i
done

# Wait for all background processes to finish
wait
