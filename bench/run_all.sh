# Define the number of servers
cargo build

SRV_LEN=$(ls -1 config/cluster_res/ | wc -l)

echo "Starting ${SRV_LEN} servers..."

# Function to run server commands
run_server() {
    local i=$1
    RUST_BACKTRACE=1 RUST_LOG=info target/debug/MerkleVerseWrapper server -c config/cluster_res/srv_${i}.toml &
    PORT=$((6000+i)) ../MerkleSquare/demo/mverserver/mverserver &
}

cleanup() {
    echo "Interrupt received, stopping servers..."
    kill $(jobs -p)  # Kills all jobs running in the background
    exit 0
}

# Trap SIGINT (Ctrl+C)
trap cleanup SIGINT

# Start all servers
for ((i=0; i<SRV_LEN; i++)); do
    run_server $i
done

# Wait for all background processes to finish
wait
