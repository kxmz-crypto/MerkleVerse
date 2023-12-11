import encoding from 'k6/encoding';
import { crypto } from 'k6/experimental/webcrypto';

export const SRV_NUM = 10;
const SRV_START = 8000;

export function get_rand_port() {
    return Math.floor(Math.random() * SRV_NUM) + SRV_START;
}

export function gen_random_loc(len){
    let randomNum = Math.floor(Math.random() * 2**Number(len));
    let buffer = Uint8Array.from([randomNum]).buffer;
    return encoding.b64encode(buffer);
}

export function gen_random_bytes(len){
    // returns a random byte array of length len
    let arr = new Uint8Array([20,31,9,5,20]);
    return encoding.b64encode(arr);
}
