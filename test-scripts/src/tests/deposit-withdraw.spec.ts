import { expect } from 'chai';
import { StarknetTestHelper } from '../helpers/starknet-test.helper';
import { SubstrateHelper } from '../helpers/substrate.helper';
import { SubstrateService } from '../providers/substrate.service';
import { assets, markets } from '../data';
import { StarknetAccountEntity, TradingAccountEntity } from '../entities';
import { config } from '../../config';

let substrateService: SubstrateService;
let tradingAccount: TradingAccountEntity;
let starknetAccount: StarknetAccountEntity;
const ethAsset = assets[0];
const usdcAsset = assets[1];
const ethUsdcMarket = markets[0];


describe('Deposit + Withdrawal', () => {
  
  before(async () => {
    substrateService = new SubstrateService({
      wsUrl: config.wsUrl,
      nodeAccount: config.nodeAccount,
    });
  
    await substrateService.initApi();
    await substrateService.replaceAssets();
    await new Promise((resolve) => setTimeout(resolve, 10000));
    await substrateService.replaceMarkets();
    await new Promise((resolve) => setTimeout(resolve, 10000));
  });

  after(async () => {
    await substrateService.disconnectApi();
  });
  
  it('Should create account', async () => {
    starknetAccount = StarknetTestHelper.generateAccount();
    tradingAccount = SubstrateHelper.generateTradingAccount({
      starkKey: starknetAccount.starkKey,
    });

    const accountResultHex = await substrateService.createAccount({
      tradingAccount
    });

    await new Promise((resolve) => setTimeout(resolve, 10000));

    expect(accountResultHex).to.match(/^0x[0-9a-f]+$/i);
  });

  it('Should check balance before deposit', async () => {
    const beforeDepositBalance = await substrateService.getBalance({
      accountId: tradingAccount.id,
      assetId: usdcAsset.id,
    });

    expect(beforeDepositBalance.value).to.equal(10000);
  });

  it('Should check balance after deposit', async () => {
    await substrateService.deposit({
      tradingAccount,
      assetId: usdcAsset.id,
      amount: 100,
    })

    await new Promise((resolve) => setTimeout(resolve, 10000));

    const afterDepositBalance = await substrateService.getBalance({
      accountId: tradingAccount.id,
      assetId: usdcAsset.id,
    });

    expect(afterDepositBalance.value).to.equal(10100);
  });

  it('Should check balance after withdrawal', async () => {
    await substrateService.withdraw({
      accountId: tradingAccount.id,
      assetId: usdcAsset.id,
      amount: 11,
      privateKey: starknetAccount.privateKey,
    });

    await new Promise((resolve) => setTimeout(resolve, 10000));

    const afterWithdrawalBalance = await substrateService.getBalance({
      accountId: tradingAccount.id,
      assetId: usdcAsset.id,
    });

    expect(afterWithdrawalBalance.value).to.equal(10089);
  });
});
