extern crate chrono;
use chrono::prelude::*;
use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Node {
    pub exchange: String,
    pub currency: String,
    pub index: Cell<usize>,
}

impl Node {
    pub fn new(exchange: &str, currency: &str, idx: usize) -> Self {
        Node {
            exchange: exchange.to_owned(),
            currency: currency.to_owned(),
            index: Cell::new(idx),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.exchange == other.exchange && self.currency == other.currency
    }
}

impl Eq for Node {}

#[derive(Debug, PartialEq, Clone)]
pub enum Factor {
    Forward,
    Backward,
    FilledUpForward,
    FilledUpBackward,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub start_node: Rc<Node>,
    pub end_node: Rc<Node>,
    pub weight: f32,
    pub timestamp: DateTime<Utc>,
    pub factor_type: Factor,
}

impl Path {
    pub fn new(
        start_node: Rc<Node>,
        end_node: Rc<Node>,
        weight: f32,
        timestamp: DateTime<Utc>,
        factor_type: Factor,
    ) -> Self {
        Path {
            start_node,
            end_node,
            weight,
            timestamp,
            factor_type,
        }
    }
}

#[derive(Debug)]
pub struct ExchangeRateRequest {
    pub source_exchange: String,
    pub source_currency: String,
    pub destination_exchange: String,
    pub destination_currency: String,
}

impl ExchangeRateRequest {
    pub fn new(
        source_exchange: String,
        source_currency: String,
        destination_exchange: String,
        destination_currency: String,
    ) -> Self {
        ExchangeRateRequest {
            source_exchange,
            source_currency,
            destination_exchange,
            destination_currency,
        }
    }
}

#[derive(Debug)]
pub struct PriceUpdateRequest {
    pub timestamp: DateTime<Utc>,
    pub exchange: String,
    pub source_currency: String,
    pub destination_currency: String,
    pub forward_factor: f32,
    pub backward_factor: f32,
}

impl PriceUpdateRequest {
    pub fn new(
        timestamp: DateTime<Utc>,
        exchange: String,
        source_currency: String,
        destination_currency: String,
        forward_factor: f32,
        backward_factor: f32,
    ) -> Self {
        PriceUpdateRequest {
            timestamp,
            exchange,
            source_currency,
            destination_currency,
            forward_factor,
            backward_factor,
        }
    }
}

impl PartialEq for PriceUpdateRequest {
    fn eq(&self, other: &PriceUpdateRequest) -> bool {
        self.timestamp == other.timestamp
            && self.exchange == other.exchange
            && self.source_currency == other.source_currency
            && self.destination_currency == other.destination_currency
            && self.forward_factor == other.forward_factor
            && self.backward_factor == other.backward_factor
    }
}

impl Eq for PriceUpdateRequest {}

#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeCurrencyPair {
    pub exchange: String,
    pub currency: String,
}

impl ExchangeCurrencyPair {
    pub fn new(exchange: String, currency: String) -> Self {
        ExchangeCurrencyPair { exchange, currency }
    }
}

#[derive(Debug, PartialEq)]
pub struct OptimalRateWithPath {
    pub rate: f32,
    pub paths: VecDeque<ExchangeCurrencyPair>,
}

impl OptimalRateWithPath {
    pub fn new(rate: f32, paths: VecDeque<ExchangeCurrencyPair>) -> Self {
        OptimalRateWithPath { rate, paths }
    }
}
