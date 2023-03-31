use parity_scale_codec::{Compact, Decode, Encode, Error, Input};
use scale_bits::{
    decode_using_format_from,
    scale::format::{Format, OrderFormat, StoreFormat},
    Bits,
};
use std::marker::PhantomData;

fn bit_format<Store: BitStore, Order: BitOrder>() -> Format {
    Format {
        order: Order::FORMAT,
        store: Store::FORMAT,
    }
}

macro_rules! store {
    ($ident: ident; $(($ty: ident, $wrapped: ty)),*) => {
        /// Associates `bitvec::store::BitStore` trait with corresponding, type-erased `scale_bits::StoreFormat` enum.
        ///
        /// Used to decode bit sequences by providing `scale_bits::StoreFormat` using
        /// `bitvec`-like type type parameters.
        pub trait $ident {
            /// Corresponding `scale_bits::StoreFormat` value.
            const FORMAT: StoreFormat;
            /// Number of bits that the backing store types holds.
            const BITS: u32;
        }

        $(
            impl $ident for $wrapped {
                const FORMAT: StoreFormat = StoreFormat::$ty;
                const BITS: u32 = <$wrapped>::BITS;
            }
        )*
    };
}

macro_rules! order {
    ($ident: ident; $($ty: ident),*) => {
        /// Associates `bitvec::order::BitOrder` trait with corresponding, type-erased `scale_bits::OrderFormat` enum.
        ///
        /// Used to decode bit sequences in runtime by providing `scale_bits::OrderFormat` using
        /// `bitvec`-like type type parameters.
        pub trait $ident {
            /// Corresponding `scale_bits::OrderFormat` value.
            const FORMAT: OrderFormat;
        }

        $(
            #[doc = concat!("Type-level value that corresponds to `scale_bits::OrderFormat::", stringify!($ty), "` at run-time")]
            #[doc = concat!(" and `bitvec::order::BitOrder::", stringify!($ty), "` at the type level.")]
            #[derive(Clone, Debug, PartialEq, Eq)]
            pub enum $ty {}
            impl $ident for $ty {
                const FORMAT: OrderFormat = OrderFormat::$ty;
            }
        )*
    };
}

store!(BitStore; (U8, u8), (U16, u16), (U32, u32), (U64, u64));
order!(BitOrder; Lsb0, Msb0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedBits<Store: BitStore, Order: BitOrder>(
    Bits,
    PhantomData<Store>,
    PhantomData<Order>,
);

impl<Store: BitStore, Order: BitOrder> DecodedBits<Store, Order> {
    /// Extracts the underlying `scale_bits::Bits` value.
    pub fn into_bits(self) -> Bits {
        self.0
    }

    /// References the underlying `scale_bits::Bits` value.
    pub fn as_bits(&self) -> &Bits {
        &self.0
    }
}

impl<Store: BitStore, Order: BitOrder> FromIterator<bool> for DecodedBits<Store, Order> {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        DecodedBits(Bits::from_iter(iter), PhantomData, PhantomData)
    }
}

impl<Store: BitStore, Order: BitOrder> Decode for DecodedBits<Store, Order> {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        /// Equivalent of `BitSlice::MAX_BITS` on 32bit machine.
        const ARCH32BIT_BITSLICE_MAX_BITS: u32 = 0x1fff_ffff;

        let Compact(bits) = <Compact<u32>>::decode(input)?;
        // Otherwise it is impossible to store it on 32bit machine.
        if bits > ARCH32BIT_BITSLICE_MAX_BITS {
            return Err("Attempt to decode a BitVec with too many bits".into());
        }
        // NOTE: Replace with `bits.div_ceil(Store::BITS)` if `int_roundings` is stabilised
        let elements = (bits / Store::BITS) + u32::from(bits % Store::BITS != 0);
        let bytes_in_elem = Store::BITS.saturating_div(u8::BITS);
        let bytes_needed = (elements * bytes_in_elem) as usize;

        // NOTE: We could reduce allocations if it would be possible to directly
        // decode from an `Input` type using a custom format (rather than default <u8, Lsb0>)
        // for the `Bits` type.
        let mut storage = Encode::encode(&Compact(bits));
        let prefix_len = storage.len();
        storage.reserve_exact(bytes_needed);
        storage.extend(vec![0; bytes_needed]);
        input.read(&mut storage[prefix_len..])?;

        let decoder = decode_using_format_from(&storage, bit_format::<Store, Order>())?;
        let bits = decoder.collect::<Result<Vec<_>, _>>()?;
        let bits = Bits::from_iter(bits);

        Ok(DecodedBits(bits, PhantomData, PhantomData))
    }
}
