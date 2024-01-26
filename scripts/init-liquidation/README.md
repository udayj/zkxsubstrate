## ENV

- NODE_ACCOUNT - Root account on Substrate. Default `//Alice`
- SUBSTRATE_WS_URL - Websocket url for connection to Substrate. Default `ws://127.0.0.1:9944`
- SIGNERS_PUB_KEYS - Websocket url for connection to Substrate. Default empty string

## Example

```sh
NODE_ACCOUNT="//Bob" SUBSTRATE_WS_URL="ws://127.0.0.1:9943" SIGNERS_PUB_KEYS="test,test2" npm start
```
