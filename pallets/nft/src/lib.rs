#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use codec::{Codec, FullCodec, MaxEncodedLen, EncodeLike};

	use sp_runtime::{
		traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize,Zero,CheckedAdd,CheckedSub}
		,ArithmeticError,FixedPointOperand,};
	use sp_std::vec::Vec;
	use sp_std::{fmt::Debug,cmp::{Eq, PartialEq}};

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

    #[pallet::pallet]
    #[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type TokenId: Parameter
			+ Member
			+ FullCodec
			+ Eq
			+ PartialEq
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ TypeInfo
			+ EncodeLike;
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo
			+ FixedPointOperand;
	}

	#[pallet::storage]
	#[pallet::getter(fn uri)]
	pub(super) type StringURI<T> = StorageValue<_, Vec<u8>,ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_tokens_count)]
	pub(super) type TokensCount<T> = StorageValue<_, u128,ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn balance_of)]
	pub(super) type Balances<T:Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn operator_approvals)]
	pub(super) type OperatorApprovals<T:Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn mint_approvals)]
	pub(super) type MintApprovals<T:Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_token_uri)]
	pub(super) type TokenURI<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		Vec<u8>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_supply)]
	pub(super) type TotalSupply<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::TokenId,
		T::Balance,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TokenMinted{to: T::AccountId,id: T::TokenId,amount: T::Balance},
		ApprovalForAll{owner: T::AccountId,operator: T::AccountId,approved: bool},
		TokenTransferred{from: T::AccountId, to: T::AccountId,id: T::TokenId, amount: T::Balance},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Setting Approval For Self
		SettingApprovalForSelf,
		/// Token Does Not Exist
		TokenDoesNotExist,
		/// Token Already Exists
		TokenAlreadyExists,
		/// Same Address
		SameAddress,
		/// Zero Amount
		ZeroAmount,
		/// Insufficient Balance For Transfer
		InsufficientBalanceForTransfer,
		/// Overflow
		Overflow,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,4).ref_time())]
		pub fn mint(origin: OriginFor<T>,to: T::AccountId,id: T::TokenId,amount: T::Balance,uri: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::token_exists(id),Error::<T>::TokenAlreadyExists);
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);
			//ensure!(to != &T::AccountId::default(), Error::<T>::ZeroAddress);
			let tokens_count = Self::get_tokens_count().checked_add(1).ok_or(ArithmeticError::Overflow)?;

			TotalSupply::<T>::insert(id,amount);
			TokenURI::<T>::insert(id,uri);
			Balances::<T>::insert(id,to.clone(),amount);
			TokensCount::<T>::put(tokens_count);
			Self::deposit_event(Event::TokenMinted { to, id, amount });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2).ref_time())]
		pub fn transfer(origin: OriginFor<T>,to: T::AccountId,id: T::TokenId,amount: T::Balance) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(!Self::token_exists(id),Error::<T>::TokenAlreadyExists);
			ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);
			ensure!(to != who, Error::<T>::SameAddress);

			Self::_transfer(who.clone(),to.clone(),id,amount)?;
			Self::deposit_event(Event::TokenTransferred{from: who, to: to,id: id, amount: amount});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,2).ref_time())]
		pub fn set_approval_for_all(origin: OriginFor<T>, operator: T::AccountId, approved: bool) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			ensure!(owner != operator, Error::<T>::SettingApprovalForSelf);
			Self::_set_approval_for_all(owner.clone(),operator.clone(),approved)?;
			Self::deposit_event(Event::ApprovalForAll{ owner,operator,approved });
			Ok(())
		}

	}

	// Helpful functions
	impl<T: Config> Pallet<T> {
		pub fn _set_approval_for_all(owner: T::AccountId, operator: T::AccountId, approved: bool) -> DispatchResult {
			OperatorApprovals::<T>::insert(owner.clone(),operator.clone(),approved);
			Ok(())
		}

		pub fn _transfer(from: T::AccountId, to: T::AccountId, id: T::TokenId, amount: T::Balance) -> DispatchResult {
			Balances::<T>::try_mutate(id.clone(),from.clone(),|balance|-> Result<(), Error<T>> {
				let from_balance =
					balance.unwrap().checked_sub(&amount).ok_or(Error::<T>::InsufficientBalanceForTransfer)?;
				*balance = Some(from_balance);
				Ok(())
			})?;
			Balances::<T>::try_mutate(id.clone(),to.clone(),|balance|-> Result<(), Error<T>> {
				let from_balance =
					balance.unwrap().checked_add(&amount).ok_or(Error::<T>::Overflow)?;
				*balance = Some(from_balance);
				Ok(())
			})?;
			Ok(())
		}

		pub fn _batch_transfer_from(from: T::AccountId, to: T::AccountId, ids: Vec<T::TokenId>, amounts: Vec<T::Balance>) -> DispatchResult {

			Ok(())
		}

		pub fn _mint(to: T::AccountId) -> DispatchResult {
			Ok(())
		}

		pub fn _mint_batch(to: T::AccountId) -> DispatchResult {
			Ok(())
		}

		pub fn token_exists(id: T::TokenId) -> bool {
			let supply = Self::get_total_supply(id).unwrap();
			if supply.is_zero() {
				false
			} else {
				true
			}
		}

		
	}
}