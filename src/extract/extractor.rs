use std::future::Future;

use http::HeaderMap;

use super::chain::chain_extractor::ChainExtractor;

use super::chain::lite::{AsyncChainLite, ChainLite};
use super::{
    and::AndExtractor,
    any::Any,
    convert::{AsyncConvert, Convert},
    map::{AsyncMap, Map, MapError},
    optional::Optional,
    or::OrExtractor,
};

pub trait Extractor {
    type Extracted: Clone + Send + Sync;

    type Error;

    fn extract(
        &self,
        headers: &HeaderMap,
    ) -> impl Future<Output = Result<Self::Extracted, Self::Error>> + Send;

    fn extracted_type_name(&self) -> &'static str {
        std::any::type_name::<Self::Extracted>()
    }
}

pub trait ExtractorExt: Sized + Extractor {
    fn map<Fn>(self, map: Fn) -> Map<Self, Fn> {
        Map::new(self, map)
    }

    fn async_map<Fn>(self, map: Fn) -> AsyncMap<Self, Fn> {
        AsyncMap::new(self, map)
    }

    fn map_err<Fn>(self, map_err: Fn) -> MapError<Self, Fn> {
        MapError::new(self, map_err)
    }

    fn convert<Fn>(self, convert: Fn) -> Convert<Self, Fn> {
        Convert::new(self, convert)
    }

    fn async_convert<Fn>(self, convert: Fn) -> AsyncConvert<Self, Fn> {
        AsyncConvert::new(self, convert)
    }

    fn chain<C>(self, chain: C) -> ChainExtractor<Self, C> {
        ChainExtractor::new(self, chain)
    }

    fn chain_lite<Fn>(self, chain: Fn) -> ChainLite<Self, Fn> {
        ChainLite::new(self, chain)
    }

    fn async_chain_lite<Fn>(self, chain: Fn) -> AsyncChainLite<Self, Fn> {
        AsyncChainLite::new(self, chain)
    }

    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }

    fn any<Ex>(self, other: Ex) -> Any<Self, Ex> {
        Any::new(self, other)
    }

    fn or<Ex>(self, other: Ex) -> OrExtractor<Self, Ex> {
        OrExtractor::new(self, other)
    }

    fn and<Ex>(self, other: Ex) -> AndExtractor<Self, Ex> {
        AndExtractor::new(self, other)
    }
}

impl<T> ExtractorExt for T where T: Sized + Extractor {}
