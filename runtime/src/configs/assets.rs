// This is free and unencumbered software released into the public domain.
//
// Anyone is free to copy, modify, publish, use, compile, sell, or
// distribute this software, either in source code form or as a compiled
// binary, for any purpose, commercial or non-commercial, and by any
// means.
//
// In jurisdictions that recognize copyright laws, the author or authors
// of this software dedicate any and all copyright interest in the
// software to the public domain. We make this dedication for the benefit
// of the public at large and to the detriment of our heirs and
// successors. We intend this dedication to be an overt act of
// relinquishment in perpetuity of all present and future rights to this
// software under copyright law.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
// OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
// ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
// OTHER DEALINGS IN THE SOFTWARE.
//
// For more information, please refer to <http://unlicense.org>

use alloc::vec;
use frame_system::EnsureRoot;
use frame_support::{derive_impl, ord_parameter_types, parameter_types, PalletId, traits::{ConstU32, ConstU128, NeverEnsureOrigin, fungible::{NativeFromLeft, NativeOrWithId, UnionOf}, tokens::imbalance::ResolveAssetTo}};
use pallet_asset_conversion::{Ascending, Chain, WithFirstAsset};
use sp_runtime::{Permill, traits::AccountIdConversion};

use crate::{Assets, AssetConversion, PoolAssets};
use super::{AccountId, Balance, Balances, ExistentialDeposit, Runtime, RuntimeEvent};

pub type ForeignAssetsInstance = pallet_assets::Instance1;
#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config<ForeignAssetsInstance> for Runtime {
    type Currency = Balances;
    type Balance = Balance;
    type AssetId = NumericAssetId;
    type AssetIdParameter = NumericAssetId;
    type CreateOrigin = NeverEnsureOrigin<AccountId>; // Only Root can create assets via `force`.
    type ForceOrigin = EnsureRoot<AccountId>;
    type AssetDeposit = ConstU128<1>;
    type AssetAccountDeposit = ConstU128<1>;
    type MetadataDepositBase = ConstU128<1>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type Freezer = ();
    type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

pub type NumericAssetId = u32;

pub type PoolAssetsInstance = pallet_assets::Instance2;
impl pallet_assets::Config<PoolAssetsInstance> for Runtime {
    type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<1000>;
	type AssetId = NumericAssetId;
	type AssetIdParameter = NumericAssetId;
	type Currency = Balances;
	type CreateOrigin = NeverEnsureOrigin<AccountId>;
	type ForceOrigin = EnsureRoot<AccountId>;
	// Deposits are zero because creation/admin is limited to Asset Conversion pallet.
	type AssetDeposit = ConstU128<0>;
	type AssetAccountDeposit = ConstU128<0>;
	type MetadataDepositBase = ConstU128<0>;
	type MetadataDepositPerByte = ConstU128<0>;
	type ApprovalDeposit = ExistentialDeposit;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Extra = ();
	type WeightInfo = ();
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

pub type NativeAndAssets = UnionOf<
	Balances,
	Assets,
	NativeFromLeft,
	NativeOrWithId<NumericAssetId>,
	AccountId,
>;

pub type PoolIdToAccountId = pallet_asset_conversion::AccountIdConverter<
	AssetConversionPalletId,
	(NativeOrWithId<NumericAssetId>, NativeOrWithId<NumericAssetId>),
>;

pub type AscendingLocator =
	Ascending<AccountId, NativeOrWithId<NumericAssetId>, PoolIdToAccountId>;

parameter_types! {
    pub const Native: NativeOrWithId<NumericAssetId> = NativeOrWithId::Native;
    pub const AssetConversionPalletId: PalletId = PalletId(*b"py/ascon");
    // we charge no fee for liquidity withdrawal
	pub const LiquidityWithdrawalFee: Permill = Permill::from_perthousand(0);
}

ord_parameter_types! {
	pub const AssetConversionOrigin: sp_runtime::AccountId32 =
		AccountIdConversion::<sp_runtime::AccountId32>::into_account_truncating(&AssetConversionPalletId::get());
}

pub type WithFirstAssetLocator = WithFirstAsset<
	Native,
	AccountId,
	NativeOrWithId<NumericAssetId>,
	PoolIdToAccountId,
>;

impl pallet_asset_conversion::Config for Runtime {
type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type HigherPrecisionBalance = sp_core::U256;
	type AssetKind = NativeOrWithId<NumericAssetId>;
	type Assets = NativeAndAssets; // The unified assets type built before.
	type PoolId = (Self::AssetKind, Self::AssetKind); // The ID for a pool is the tuple of both assets that go in it.
	type PoolLocator = Chain<WithFirstAssetLocator, AscendingLocator>;
	type PoolAssetId = u32;
	type PoolAssets = PoolAssets; // The instance of the assets pallet.
	type PoolSetupFee = ConstU128<0>; // Asset class deposit fees are sufficient to prevent spam
	type PoolSetupFeeAsset = Native;
	type PoolSetupFeeTarget = ResolveAssetTo<AssetConversionOrigin, Self::Assets>;
	type LiquidityWithdrawalFee = LiquidityWithdrawalFee;
	type LPFee = ConstU32<3>; // 0.3% swap fee
	type PalletId = AssetConversionPalletId;
	type MaxSwapPathLength = ConstU32<3>;
	type MintMinLiquidity = ConstU128<100>;
	type WeightInfo = (); // Make sure to benchmark this for production!
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl pallet_asset_conversion_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetId = NativeOrWithId<NumericAssetId>;
	type OnChargeAssetTransaction = pallet_asset_conversion_tx_payment::SwapAssetAdapter<
		Native,
		NativeAndAssets,
		AssetConversion,
		(), // Fees are burnt. You could use `ResolveAssetTo<TreasuryAccount, NativeAndAssets>` if your chain has a treasury.
	>;
	type WeightInfo = (); // Make sure to benchmark this for production!
}
