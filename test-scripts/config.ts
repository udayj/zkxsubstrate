import * as dotenv from 'dotenv';

dotenv.config(); 

const {
    wsUrl = 'ws://127.0.0.1:9944',
    nodeAccount = '//Alice',
} = process.env;

export const config = {
    wsUrl,
    nodeAccount,
}