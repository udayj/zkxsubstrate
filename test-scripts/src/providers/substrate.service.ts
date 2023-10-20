import * as baseStarknet from 'starknet';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { Keyring } from '@polkadot/keyring';
import { KeyringPair } from '@polkadot/keyring/types';
import { u8aToBn, bnToU8a } from '@polkadot/util';
import { HexString } from '@polkadot/util/types';

import { SubstrateHelper } from '../helpers';
import { AssetEntity, BalanceEntity, MarketEntity, TradingAccountEntity } from '../entities';
import { assets, markets } from '../data';
import { rpc, types } from './rpc';

const { computeHashOnElements } = baseStarknet.hash;
const { sign } = baseStarknet.ec.starkCurve;
const { toHex } = baseStarknet.num;

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
    const assetListInRaw = assets.map(asset => {
      return {
        asset: {
          id: SubstrateHelper.convertStringToU128(asset.id),
          version: asset.version,
          short_name: SubstrateHelper.convertStringToU256(asset.shortName),
          is_tradable: asset.isTradable,
          is_collateral: asset.isCollateral,
          l2_address: SubstrateHelper.convertHexToU256(asset.l2Address),
          decimals: asset.decimals,
        },
        metadata_url: asset.metadataUrl,
      }
    });

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );

    await this.wsApi.tx.assets
      .replaceAllAssets(assetListInRaw)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });
  }

  async replaceMarkets(): Promise<void> {
    const marketListInRaw = markets.map(market => {
      return {
        market: {
          id: SubstrateHelper.convertStringToU128(market.id),
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
        },
        metadata_url: market.metadataUrl,
      };
    });

    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );
    
    await this.wsApi.tx.markets
      .replaceAllMarkets(marketListInRaw)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });
  }

  async getAssets(): Promise<AssetEntity[]> {
    const assetMap = await this.wsApi.query.assets.assetMap.entries();

    const assetList = assetMap.map(([, raw]): AssetEntity => {
      const { asset, metadataUrl } = raw.toPrimitive() as any;

      return new AssetEntity({
        id: SubstrateHelper.convertU128ToString(asset.id),
        version: asset.version,
        shortName: SubstrateHelper.convertU256ToString(asset.shortName),
        isTradable: asset.isTradable,
        isCollateral: asset.isCollateral,
        l2Address: SubstrateHelper.convertU256ToHex(asset.l2Address),
        decimals: asset.decimals,
        metadataUrl,
      });
    });

    return assetList;
  }

  async getMarkets(): Promise<MarketEntity[]> {
    const marketMap = await this.wsApi.query.markets.marketMap.entries();

    const marketList = marketMap.map(([, raw]): MarketEntity => {
      const { market, metadataUrl } = raw.toPrimitive() as any;

      return new MarketEntity({
        id: SubstrateHelper.convertU128ToString(market.id),
        asset: SubstrateHelper.convertU128ToString(market.asset),
        assetCollateral: SubstrateHelper.convertU128ToString(
          market.assetCollateral,
        ),
        isTradable: market.isTradable === true,
        isArchived: market.isArchived === true,
        ttl: market.ttl,
        tickSize: SubstrateHelper.convertI128ToNumber(market.tickSize),
        tickPrecision: market.tickPrecision,
        stepSize: SubstrateHelper.convertI128ToNumber(market.stepSize),
        stepPrecision: market.stepPrecision,
        minimumOrderSize: SubstrateHelper.convertI128ToNumber(
          market.minimumOrderSize,
        ),
        minimumLeverage: SubstrateHelper.convertI128ToNumber(
          market.minimumLeverage,
        ),
        maximumLeverage: SubstrateHelper.convertI128ToNumber(
          market.maximumLeverage,
        ),
        currentlyAllowedLeverage: SubstrateHelper.convertI128ToNumber(
          market.currentlyAllowedLeverage,
        ),
        maintenanceMarginFraction: SubstrateHelper.convertI128ToNumber(
          market.maintenanceMarginFraction,
        ),
        initialMarginFraction: SubstrateHelper.convertI128ToNumber(
          market.initialMarginFraction,
        ),
        incrementalInitialMarginFraction: SubstrateHelper.convertI128ToNumber(
          market.incrementalInitialMarginFraction,
        ),
        incrementalPositionSize: SubstrateHelper.convertI128ToNumber(
          market.incrementalPositionSize,
        ),
        baselinePositionSize: SubstrateHelper.convertI128ToNumber(
          market.baselinePositionSize,
        ),
        maximumPositionSize: SubstrateHelper.convertI128ToNumber(
          market.maximumPositionSize,
        ),
        metadataUrl,
      });
    });

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
    privateKey: string;
  }): Promise<string> {
    const { accountId, assetId, amount, privateKey } = params;

    const withdrawalRequest: any = {
      account_id: SubstrateHelper.convertHexToU256(accountId),
      collateral_id: SubstrateHelper.convertStringToU128(assetId),
      amount: SubstrateHelper.convertNumberToI128(amount),
      hash_type: 0,
      sig_r: null,
      sig_s: null,
    }

    const accountIdAsBytes = bnToU8a(withdrawalRequest.account_id.toPrimitive(), { isLe: false });
    const accountIdAsLowBytes = accountIdAsBytes.slice(16);
    const accountIdAsHighBytes = accountIdAsBytes.slice(0, 16);

    const withdrawHashElements = [
      u8aToBn(accountIdAsLowBytes, { isLe: false }).toString(),
      u8aToBn(accountIdAsHighBytes, { isLe: false }).toString(),
      withdrawalRequest.collateral_id.toPrimitive(),
      withdrawalRequest.amount.toPrimitive(),
    ];

    const dataHash = computeHashOnElements(withdrawHashElements);

    const signature = sign(dataHash, privateKey);
    const sigR = toHex(signature.r);
    const sigS = toHex(signature.s);

    withdrawalRequest.sig_r = sigR;
    withdrawalRequest.sig_s = sigS;
    
    const nonce = await this.wsApi.rpc.system.accountNextIndex(
      this.nodeAccountKeyring.address,
    );

    const withdrawResult = await this.wsApi.tx.zkxTradingAccount
      .withdraw(withdrawalRequest)
      .signAndSend(this.nodeAccountKeyring, {
        nonce,
      });

    const withdrawResultHex = withdrawResult.toHex();

    return withdrawResultHex;
  }
}
