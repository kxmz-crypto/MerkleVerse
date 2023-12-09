from pathlib import Path
import subprocess
import threading

SRV_LEN = 5

def run_command(command):
    process = subprocess.Popen(command, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    # Wait for the command to complete
    process.wait()
    print(process)

def run_server(i):
    mverse_cmd = f"RUST_BACKTRACE=1 RUST_LOG=debug target/debug/MerkleVerseWrapper -c config/cluster_res/srv_{i}.toml"
    msq_cmd = f"PORT={6000+i} ../MerkleSquare/demo/mverserver"

    # Create threads that run the commands and wait for them to complete
    thread1 = threading.Thread(target=run_command, args=(mverse_cmd,))
    thread2 = threading.Thread(target=run_command, args=(msq_cmd,))

    return [thread1, thread2]

def start_all():
    threads = []
    for i in range(SRV_LEN):
        threads.extend(run_server(i))
    for t in threads:
        t.start()
    return threads

def main():
    threads = start_all()
    # Wait for all threads to complete
    for t in threads:
        t.join()

if __name__ == "__main__":
    main()
