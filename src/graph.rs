use crate::domain::*;
use crate::error::*;
use chrono::prelude::*;
use std::collections::VecDeque;
use std::rc::Rc;

pub type GraphResult = Result<OptimalRateWithPath, GraphError>;

#[derive(Debug)]
pub struct Graph {
    nodes: Vec<Rc<Node>>,
    paths: Vec<Path>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: Vec::<Rc<Node>>::new(),
            paths: Vec::<Path>::new(),
        }
    }

    pub fn get_top_result(&self, exchange_request: &ExchangeRateRequest) -> GraphResult {
        let start_node = self.find_node_with(
            &exchange_request.source_exchange,
            &exchange_request.source_currency,
        );

        let end_node = self.find_node_with(
            &exchange_request.destination_exchange,
            &exchange_request.destination_currency,
        );

        match (start_node, end_node) {
            (Some(sn), Some(en)) => {
                let paths = self.get_top_paths(&sn, &en);
                if paths.is_empty() {
                    Err(GraphError::PathNotFound)
                } else if paths.len() == 1 {
                    // check for the exchange rate request like "KRAKEN, BTC, KRAKEN, BTC"
                    Err(GraphError::InvalidPath)
                } else {
                    let mut rate = 1_f32;
                    let mut pair: VecDeque<ExchangeCurrencyPair> =
                        VecDeque::with_capacity(paths.len());

                    let mut iter = paths.windows(2);
                    while let Some(&[si, ei]) = iter.next() {
                        let path = self.paths.iter().find(|p| {
                            p.start_node.index.get() == si && p.end_node.index.get() == ei
                        });
                        if path.is_none() {
                            return Err(GraphError::Critical);
                        }

                        let path = path.unwrap();
                        rate *= path.weight;
                        pair.push_back(ExchangeCurrencyPair::new(
                            path.start_node.exchange.clone(),
                            path.start_node.currency.clone(),
                        ));
                    }

                    pair.push_back(ExchangeCurrencyPair::new(
                        en.exchange.clone(),
                        en.currency.clone(),
                    ));
                    Ok(OptimalRateWithPath::new(rate, pair))
                }
            }
            _ => Err(GraphError::PathNotFound),
        }
    }

    fn get_top_paths(&self, start_node: &Rc<Node>, end_node: &Rc<Node>) -> Vec<usize> {
        let next = self.reconstruct_path();
        let mut paths = vec![];
        let mut u = start_node.index.get();
        let v = end_node.index.get();
        if next[u][v].is_some() {
            paths.push(u);
            while u != v {
                u = next[u][v].unwrap();
                paths.push(u);
            }
        }
        paths
    }

    fn reconstruct_path(&self) -> Vec<Vec<Option<usize>>> {
        let mut rate: Vec<Vec<f32>> = Vec::with_capacity(self.paths.len());
        let mut next: Vec<Vec<Option<usize>>> = Vec::with_capacity(self.paths.len());

        let len = self.paths.len();

        for i in 0..len {
            rate.push(Vec::with_capacity(len));
            next.push(Vec::with_capacity(len));
            for _ in 0..len {
                rate[i].push(0.0);
                next[i].push(None);
            }
        }

        for i in 0..len {
            let from_index = self.paths[i].start_node.index.get();
            let to_index = self.paths[i].end_node.index.get();
            rate[from_index][to_index] = self.paths[i].weight;
            next[from_index][to_index] = Some(to_index);
        }

        for k in 0..len {
            for i in 0..len {
                for j in 0..len {
                    if rate[i][j] < rate[i][k] * rate[k][j] {
                        rate[i][j] = rate[i][k] * rate[k][j];
                        next[i][j] = next[i][k];
                    }
                }
            }
        }

        next
    }

    pub fn update(&mut self, request: &PriceUpdateRequest) {
        let start_node = Rc::new(Node::new(&request.exchange, &request.source_currency, 0));

        let end_node = Rc::new(Node::new(
            &request.exchange,
            &request.destination_currency,
            0,
        ));

        let existing_paths: Vec<(Factor, &mut Path)> = self
            .paths
            .iter_mut()
            .filter_map(|p| {
                let mut t: Option<(Factor, &mut Path)> = None;
                if p.start_node.eq(&start_node) && p.end_node.eq(&end_node) {
                    t = Some((Factor::Forward, p));
                } else if p.start_node.eq(&end_node) && p.end_node.eq(&start_node) {
                    t = Some((Factor::Backward, p));
                }
                t
            })
            .collect();

        if !existing_paths.is_empty() {
            // update existing paths
            for p in existing_paths {
                if p.0 == Factor::Forward && request.timestamp > p.1.timestamp {
                    p.1.timestamp = request.timestamp;
                    p.1.weight = request.forward_factor;
                }
                if p.0 == Factor::Backward && request.timestamp > p.1.timestamp {
                    p.1.timestamp = request.timestamp;
                    p.1.weight = request.backward_factor;
                }
            }
        } else {
            // insert new paths
            self.paths.push(Path::new(
                Rc::clone(&start_node),
                Rc::clone(&end_node),
                request.forward_factor,
                request.timestamp,
                Factor::Forward,
            ));

            self.paths.push(Path::new(
                Rc::clone(&end_node),
                Rc::clone(&start_node),
                request.backward_factor,
                request.timestamp,
                Factor::Backward,
            ));

            // create new paths with weight 1.0 if new exchange
            self.insert_additional_paths(
                Rc::clone(&start_node),
                Rc::clone(&end_node),
                request.timestamp,
            );
        }

        // Insert if new node
        self.insert_node(&start_node);
        self.insert_node(&end_node);
    }

    fn insert_additional_paths(
        &mut self,
        start_node: Rc<Node>,
        end_node: Rc<Node>,
        ts: DateTime<Utc>,
    ) {
        for existing_node in self.nodes.iter() {
            if !self.nodes.contains(&start_node)
                && existing_node.currency == start_node.currency
                && existing_node.exchange != start_node.exchange
            {
                self.paths.push(Path::new(
                    Rc::clone(existing_node),
                    Rc::clone(&start_node),
                    1.0,
                    ts,
                    Factor::FilledUpForward,
                ));
                self.paths.push(Path::new(
                    Rc::clone(&start_node),
                    Rc::clone(existing_node),
                    1.0,
                    ts,
                    Factor::FilledUpBackward,
                ));
            } else if !self.nodes.contains(&end_node)
                && existing_node.currency == end_node.currency
                && existing_node.exchange != end_node.exchange
            {
                self.paths.push(Path::new(
                    Rc::clone(existing_node),
                    Rc::clone(&end_node),
                    1.0,
                    ts,
                    Factor::FilledUpForward,
                ));
                self.paths.push(Path::new(
                    Rc::clone(&end_node),
                    Rc::clone(existing_node),
                    1.0,
                    ts,
                    Factor::FilledUpBackward,
                ));
            }
        }
    }

    fn insert_node(&mut self, n: &Rc<Node>) {
        if !self.nodes.contains(n) {
            n.index.set(self.nodes.len());
            self.nodes.push(Rc::clone(n));
        }
    }

    fn find_node_with<'a>(&self, exg: &'a str, curr: &'a str) -> Option<Rc<Node>> {
        self.nodes
            .iter()
            .find(|&n| n.exchange == exg && n.currency == curr)
            .and_then(|m| Some(Rc::clone(m)))
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.paths.clear();
    }

    pub fn get_nodes(&self) -> Vec<Node> {
        self.nodes
            .iter()
            .map(|x| (&**x).clone())
            .collect::<Vec<Node>>()
    }

    pub fn get_paths(&self) -> Vec<Path> {
        self.paths.to_vec()
    }

    pub fn get_forward_backward_factor_of_existing_paths(&self) -> (Vec<f32>, Vec<f32>) {
        let mut fnum = vec![];
        let mut bnum = vec![];
        for p in self.paths.iter() {
            if p.factor_type == Factor::Forward {
                fnum.push(p.weight);
            } else if p.factor_type == Factor::Backward {
                bnum.push(p.weight);
            }
        }
        (fnum, bnum)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utility::*;
    use std::collections::VecDeque;

    const KRAKEN_PRICE: &[&str] = &[
        "2017-11-01T09:42:23+00:00 ",
        "KRAKEN",
        "BtC",
        "UsD",
        "1000.0 ",
        "0.0009 ",
    ];
    const KRAKEN_PRICE_WITH_LATEST_DATE: &[&str] = &[
        "2018-11-01T09:42:23+00:00 ",
        "KRAKEN",
        "BtC",
        "UsD",
        "1018.0 ",
        "0.0001 ",
    ];
    const GDAX_PRICE: &[&str] = &[
        " 2017-11-01T09:42:23+00:00 ",
        " GDAX",
        " BtC",
        " UsD",
        " 1001.0 ",
        " 0.0008 ",
    ];
    const BITTREX_PRICE: &[&str] = &[
        " 2017-11-01T09:42:23+00:00 ",
        " BITTREX",
        " BtC",
        " UsD",
        " 1002.0 ",
        " 0.0009 ",
    ];

    #[test]
    fn with_correct_two_exchange_data() {
        let mut g = Graph::new();
        let kraken = validate_price_update_input(&KRAKEN_PRICE, &g);
        g.update(&kraken.unwrap());
        let gdax = validate_price_update_input(&GDAX_PRICE, &g);
        g.update(&gdax.unwrap());
        let rate_req = ExchangeRateRequest::new(
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
            "GDAX".to_owned(),
            "USD".to_owned(),
        );

        let result = g.get_top_result(&rate_req);

        let expected = OptimalRateWithPath {
            rate: 1001.0,
            paths: {
                let mut vd = VecDeque::new();
                vd.push_back(ExchangeCurrencyPair::new(
                    "KRAKEN".to_owned(),
                    "BTC".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "GDAX".to_owned(),
                    "BTC".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "GDAX".to_owned(),
                    "USD".to_owned(),
                ));
                vd
            },
        };

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn with_correct_three_exchange_data() {
        let mut g = Graph::new();
        let kraken = validate_price_update_input(&KRAKEN_PRICE, &g);
        g.update(&kraken.unwrap());
        let gdax = validate_price_update_input(&GDAX_PRICE, &g);
        g.update(&gdax.unwrap());
        let bittrex = validate_price_update_input(&BITTREX_PRICE, &g);
        g.update(&bittrex.unwrap());
        let rate_req = ExchangeRateRequest::new(
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
            "GDAX".to_owned(),
            "USD".to_owned(),
        );

        let result = g.get_top_result(&rate_req);

        let expected = OptimalRateWithPath {
            rate: 1002.0,
            paths: {
                let mut vd = VecDeque::new();
                vd.push_back(ExchangeCurrencyPair::new(
                    "KRAKEN".to_owned(),
                    "BTC".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "BITTREX".to_owned(),
                    "BTC".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "BITTREX".to_owned(),
                    "USD".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "GDAX".to_owned(),
                    "USD".to_owned(),
                ));
                vd
            },
        };

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn with_same_exchange_but_updated_data() {
        let mut g = Graph::new();
        let kraken = validate_price_update_input(&KRAKEN_PRICE, &g);
        g.update(&kraken.unwrap());
        let kraken_update = validate_price_update_input(&KRAKEN_PRICE_WITH_LATEST_DATE, &g);
        g.update(&kraken_update.unwrap());
        let gdax = validate_price_update_input(&GDAX_PRICE, &g);
        g.update(&gdax.unwrap());
        let rate_req = ExchangeRateRequest::new(
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
            "GDAX".to_owned(),
            "USD".to_owned(),
        );

        let result = g.get_top_result(&rate_req);

        let expected = OptimalRateWithPath {
            rate: 1018.0,
            paths: {
                let mut vd = VecDeque::new();
                vd.push_back(ExchangeCurrencyPair::new(
                    "KRAKEN".to_owned(),
                    "BTC".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "KRAKEN".to_owned(),
                    "USD".to_owned(),
                ));
                vd.push_back(ExchangeCurrencyPair::new(
                    "GDAX".to_owned(),
                    "USD".to_owned(),
                ));
                vd
            },
        };

        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn should_throw_path_not_found_error() {
        let mut g = Graph::new();
        let kraken = validate_price_update_input(&KRAKEN_PRICE, &g);
        let rate_req = ExchangeRateRequest::new(
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
            "GDAX".to_owned(),
            "USD".to_owned(),
        );
        g.update(&kraken.unwrap());
        let result = g.get_top_result(&rate_req);
        assert_eq!(result.unwrap_err(), GraphError::PathNotFound);
    }

    #[test]
    fn should_throw_invalid_path_error() {
        let mut g = Graph::new();
        let kraken = validate_price_update_input(&KRAKEN_PRICE, &g);
        g.update(&kraken.unwrap());
        let gdax = validate_price_update_input(&GDAX_PRICE, &g);
        g.update(&gdax.unwrap());
        let rate_req = ExchangeRateRequest::new(
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
            "KRAKEN".to_owned(),
            "BTC".to_owned(),
        );
        let result = g.get_top_result(&rate_req);
        assert_eq!(result.unwrap_err(), GraphError::InvalidPath);
    }
}
