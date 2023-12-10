import grpc from "k6/net/grpc";
import { check, sleep } from 'k6';

const client = new grpc.Client();
client.load(['../proto', '../'], 'outer.proto')
//
export const options = {
  vus: 10,
  duration: '30s',
};

// The function that defines VU logic.
//
// See https://grafana.com/docs/k6/latest/examples/get-started-with-k6/ to learn more
// about authoring k6 scripts.
//
export default function() {
  client.connect('localhost:8000', {plaintext: true});
  const data = {};
  const response = client.invoke('mverseouter.MerkleVerse/GetCurrentRoot', data);

  check(response, {
    'status is OK': (r) => r && r.status === grpc.StatusOK,
  });

  console.log(JSON.stringify(response.message));
  client.close();
  sleep(1);
}
