import grpc from "k6/net/grpc";
import { check, sleep } from 'k6';
import {gen_random_bytes, gen_random_loc, get_rand_port} from "./utils.js";

const client = new grpc.Client();
client.load(['../proto', '../'], 'outer.proto')
//
export const options = {
  vus: 8,
  duration: '60s',
};

export default function() {
  client.connect(`localhost:${get_rand_port()}`, {plaintext: true});
  let rand_loc = gen_random_loc(32);
  let rand_bytes = gen_random_bytes(5);
  const data = {
    "auxiliary": "6OWjvfz3RRkoQ7O7RdoPm1O5",
    "transaction": {
        "key": rand_loc,
        "transaction_type": "Update",
        "value": rand_bytes
    },
    "wait": true
  };

  console.log(JSON.stringify(data));

  const response = client.invoke('mverseouter.MerkleVerse/ClientTransaction', data);

  check(response, {
    'status is OK': (r) => r && r.status === grpc.StatusOK,
  });

  console.log(JSON.stringify(response));
  client.close();
  sleep(0.5);
}
