import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { stringToHex, u8aToBn, bnToU8a } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';
import { SubstrateHelper } from '../helpers/substrate.helper';
import { AssetEntity, BalanceEntity, MarketEntity, TradingAccountEntity } from '../entities';
import { assets, markets } from '../data';
import { StarknetTestHelper } from '../helpers/starknet-test.helper';
import { rpc, types } from './rpc';

const keyring = new Keyring({ type: 'sr25519' });

interface ISubstrateService {
  wsUrl: string;
  nodeAccount: string;
  maxTransactionRetries?: number
}

export class SubstrateService {
  wsUrl: string;
  nodeAccount: string;
  wsProvider: WsProvider;
  nodeAccountKeyring: KeyringPair;
  wsApi: ApiPromise;
  maxTransactionRetries: number;

  constructor(data: ISubstrateService) {
    this.wsUrl = data.wsUrl;
    this.nodeAccount = data.nodeAccount;
    this.maxTransactionRetries = data.maxTransactionRetries || 10;
  }

  async initApi() {
    await cryptoWaitReady();
    const nodeAccountKeyring = keyring.addFromUri(this.nodeAccount);
    const wsProvider = new WsProvider(this.wsUrl);
    
    const wsApi = await ApiPromise.create({
      provider: wsProvider,
      types,
      rpc,
    });

    this.wsProvider = wsProvider;
    this.nodeAccountKeyring = nodeAccountKeyring;
    this.wsApi = wsApi;
  }

  async disconnectApi() {
    if (this.wsProvider?.isConnected) {
      // disconnecting the underlying provider
      await this.wsProvider.disconnect();
    }
  }

  async createAccount(params: { tradingAccount: TradingAccountEntity }): Promise<HexString> {
    const { tradingAccount } = params;

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );

    const addAccountsResult = await this.wsApi.tx.zkxTradingAccount
      .addAccounts([
        {
          index: tradingAccount.index,
          account_address: SubstrateHelper.convertHexToU256(tradingAccount.address),
          pub_key: SubstrateHelper.convertHexToU256(tradingAccount.publicKey),
        },
      ])
      .signAndSend(this.nodeAccountKeyring, {
          nonce,
      });

    const addAccountsResultHex = addAccountsResult.toHex();

    return addAccountsResultHex;
  }

  async replaceAssets(): Promise<void> {
    const assetsConvertedData = assets.map(asset => ({
      id: SubstrateHelper.convertStringToU256(asset.id),
      name: stringToHex(asset.name),
      is_tradable: asset.isTradable,
      is_collateral: asset.isCollateral,
      token_decimal: asset.tokenDecimal,
    }));

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );

    await this.wsApi.tx.assets
      .replaceAllAssets(assetsConvertedData)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });
  }

  async replaceMarkets(): Promise<void> {
    const marketsConvertedData = markets.map(market => ({
      id: SubstrateHelper.convertStringToU256(market.id),
      asset: SubstrateHelper.convertStringToU128(market.asset),
      asset_collateral: SubstrateHelper.convertStringToU128(
        market.assetCollateral,
      ),
      is_tradable: market.isTradable,
      is_archived: market.isArchived,
      ttl: market.ttl,
      tick_size: SubstrateHelper.convertNumberToI128(market.tickSize),
      tick_precision: market.tickPrecision,
      step_size: SubstrateHelper.convertNumberToI128(market.stepSize),
      step_precision: market.stepPrecision,
      minimum_order_size: SubstrateHelper.convertNumberToI128(
        market.minimumOrderSize,
      ),
      minimum_leverage: SubstrateHelper.convertNumberToI128(
        market.minimumLeverage,
      ),
      maximum_leverage: SubstrateHelper.convertNumberToI128(
        market.maximumLeverage,
      ),
      currently_allowed_leverage: SubstrateHelper.convertNumberToI128(
        market.currentlyAllowedLeverage,
      ),
      maintenance_margin_fraction: SubstrateHelper.convertNumberToI128(
        market.maintenanceMarginFraction,
      ),
      initial_margin_fraction: SubstrateHelper.convertNumberToI128(
        market.initialMarginFraction,
      ),
      incremental_initial_margin_fraction: SubstrateHelper.convertNumberToI128(
        market.incrementalInitialMarginFraction,
      ),
      incremental_position_size: SubstrateHelper.convertNumberToI128(
        market.incrementalPositionSize,
      ),
      baseline_position_size: SubstrateHelper.convertNumberToI128(
        market.baselinePositionSize,
      ),
      maximum_position_size: SubstrateHelper.convertNumberToI128(
        market.maximumPositionSize,
      ),
    }));

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );
    
    await this.wsApi.tx.markets
      .replaceAllMarkets(marketsConvertedData)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });
  }

  async getAssets(): Promise<AssetEntity[]> {
    const assetMap = await this.wsApi.query.assets.assetMap.entries();

    const assetList = assetMap.map(([assetKey, assetItem]): AssetEntity => {
      const data = assetItem.toPrimitive() as any;

      data.id = SubstrateHelper.convertU256ToString(data.id);

      return new AssetEntity(data);
    });

    return assetList;
  }

  async getMarkets(): Promise<MarketEntity[]> {
    const marketMap = await this.wsApi.query.markets.marketMap.entries();

    const marketList = marketMap.map(
      ([marketKey, marketItem]): MarketEntity => {
        const data = marketItem.toPrimitive() as any;

        return new MarketEntity({
          id: SubstrateHelper.convertU256ToString(data.id),
          asset: SubstrateHelper.convertU128ToString(data.asset),
          assetCollateral: SubstrateHelper.convertU128ToString(
            data.assetCollateral,
          ),
          isTradable: data.isTradable === true,
          isArchived: data.isArchived === true,
          ttl: data.ttl,
          tickSize: SubstrateHelper.convertI128ToNumber(data.tickSize),
          tickPrecision: data.tickPrecision,
          stepSize: SubstrateHelper.convertI128ToNumber(data.stepSize),
          stepPrecision: data.stepPrecision,
          minimumOrderSize: SubstrateHelper.convertI128ToNumber(
            data.minimumOrderSize,
          ),
          minimumLeverage: SubstrateHelper.convertI128ToNumber(
            data.minimumLeverage,
          ),
          maximumLeverage: SubstrateHelper.convertI128ToNumber(
            data.maximumLeverage,
          ),
          currentlyAllowedLeverage: SubstrateHelper.convertI128ToNumber(
            data.currentlyAllowedLeverage,
          ),
          maintenanceMarginFraction: SubstrateHelper.convertI128ToNumber(
            data.maintenanceMarginFraction,
          ),
          initialMarginFraction: SubstrateHelper.convertI128ToNumber(
            data.initialMarginFraction,
          ),
          incrementalInitialMarginFraction: SubstrateHelper.convertI128ToNumber(
            data.incrementalInitialMarginFraction,
          ),
          incrementalPositionSize: SubstrateHelper.convertI128ToNumber(
            data.incrementalPositionSize,
          ),
          baselinePositionSize: SubstrateHelper.convertI128ToNumber(
            data.baselinePositionSize,
          ),
          maximumPositionSize: SubstrateHelper.convertI128ToNumber(
            data.maximumPositionSize,
          ),
        });
      },
    );

    return marketList;
  }

  async getBalance(params: {
    accountId: string;
    assetId: string;
  }): Promise<BalanceEntity> {
    const balanceInCodec = await this.wsApi.query.zkxTradingAccount.balancesMap(
      SubstrateHelper.convertHexToU256(params.accountId),
      SubstrateHelper.convertStringToU128(params.assetId),
    );

    return {
      assetId: params.assetId,
      value: SubstrateHelper.convertI128ToNumber(balanceInCodec.toPrimitive()),
    };
  }

  async deposit(params: {
    tradingAccount: TradingAccountEntity;
    assetId: string;
    amount: number;
  }): Promise<string> {
    const { tradingAccount, assetId, amount } = params;

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );
    
    const depositResult = await this.wsApi.tx.zkxTradingAccount
      .deposit(
        {
          account_address: tradingAccount.address,
          pub_key: tradingAccount.publicKey,
          index: tradingAccount.index
        },
        SubstrateHelper.convertStringToU128(assetId),
        SubstrateHelper.convertNumberToI128(amount),
      )
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });

    const depositResultHex = depositResult.toHex();
    
    return depositResultHex;
  }

  async withdraw(params: {
    accountId: string;
    assetId: string;
    amount: number;
    hashType: number;
    privateKey: string;
  }): Promise<string> {
    const { accountId, assetId, amount, hashType, privateKey } = params;

    const requestObject: any = {
      account_id: SubstrateHelper.convertHexToU256(accountId),
      collateral_id: SubstrateHelper.convertStringToU128(assetId),
      amount: SubstrateHelper.convertNumberToI128(amount),
      hash_type: 0,
      sig_r: null,
      sig_s: null,
    }

    const accountIdAsBytes = bnToU8a(requestObject.account_id.toPrimitive(), { isLe: false });
    const accountIdAsLowBytes = accountIdAsBytes.slice(16);
    const accountIdAsHighBytes = accountIdAsBytes.slice(0, 16);

    const withdrawHashElements = [
      u8aToBn(accountIdAsLowBytes, { isLe: false }).toString(),
      u8aToBn(accountIdAsHighBytes, { isLe: false }).toString(),
      requestObject.collateral_id.toPrimitive(),
      requestObject.amount.toPrimitive(),
      requestObject.hash_type,
    ];

    const [signR, signS] = StarknetTestHelper.sign({
      privateKey,
      data: withdrawHashElements,
    });

    requestObject.sig_r = signR;
    requestObject.sig_s = signS;
    
    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );

    const withdrawResult = await this.wsApi.tx.zkxTradingAccount
      .withdraw(requestObject)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });

    const withdrawResultHex = withdrawResult.toHex();
    
    return withdrawResultHex;
  }
}
