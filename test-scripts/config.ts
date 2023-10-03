import * as dotenv from 'dotenv';

dotenv.config(); 

const {
    WS_URL = 'ws://127.0.0.1:9944',
    NODE_ACCOUNT = '//Alice',
} = process.env;

export const config = {
    wsUrl: WS_URL,
    nodeAccount: NODE_ACCOUNT,
}