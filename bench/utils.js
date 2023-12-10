export const SRV_NUM = 20;
const SRV_START = 8000;

export function get_rand_port() {
    return Math.floor(Math.random() * SRV_NUM) + SRV_START;
}