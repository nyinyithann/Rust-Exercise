extern crate chrono;
extern crate colored;

use crate::domain::*;
use crate::error::*;
use crate::graph::*;
use chrono::prelude::*;

pub type PriceUpdateRequestValidationResult =
    Result<PriceUpdateRequest, PriceUpdateRequestValidationError>;
pub type ExchangeRateRequestValidationResult =
    Result<ExchangeRateRequest, ExchangeRateRequestValidationError>;

pub fn validate_exchange_rate_input(args: &[&str]) -> ExchangeRateRequestValidationResult {
    if args.len() != 4 {
        Err(ExchangeRateRequestValidationError::InvalidArgumentNumber)
    } else {
        Ok(ExchangeRateRequest::new(
            args[0].trim().to_uppercase(),
            args[1].trim().to_uppercase(),
            args[2].trim().to_uppercase(),
            args[3].trim().to_uppercase(),
        ))
    }
}

pub fn validate_price_update_input(args: &[&str], g: &Graph) -> PriceUpdateRequestValidationResult {
    if args.len() != 6 {
        Err(PriceUpdateRequestValidationError::InvalidArgumentNumber)
    } else if args[2].trim().to_uppercase() == args[3].trim().to_uppercase() {
        Err(PriceUpdateRequestValidationError::SameSourceDestinationCurrency)
    } else {
        DateTime::parse_from_rfc3339(args[0].trim())
        .map_err(|_| PriceUpdateRequestValidationError::InvalidTimestamp)
        .and_then(|dt| {
            args[4]
                .trim()
                .parse::<f32>()
                .map_err(|_| PriceUpdateRequestValidationError::InvalidForwardfactor)
                .map(|x| {
                    if x <= 0.0 {
                        Err(PriceUpdateRequestValidationError::InvalidForwardfactor)
                    } else {
                        Ok(x)
                    }
                })
                .and_then(|rff| {
                    args[5]
                        .trim()
                        .parse::<f32>()
                        .map_err(|_| PriceUpdateRequestValidationError::InvalidBackwardfactor)
                        .map(|x| {
                            if x <= 0.0 {
                                Err(PriceUpdateRequestValidationError::InvalidBackwardfactor)
                            } else {
                                Ok(x)
                            }
                        })
                        .and_then(|rbf| {
                            let ff = rff.unwrap();
                            let bf = rbf.unwrap();

                            let existing_paths = g.get_forward_backward_factor_of_existing_paths();
                            let ff_over_one = existing_paths.1.iter().any(|x| x * ff > 1.0);
                            let bb_over_one = existing_paths.0.iter().any(|x| x * bf > 1.0);

                            if ff_over_one || bb_over_one{
                                Err(PriceUpdateRequestValidationError::CrossForwardBackwardFactorMultiplyError)
                            }
                            else if ff * bf > 1.0 {
                                Err(PriceUpdateRequestValidationError::ForwardBackwardFactorMultiplyError)
                            }                            
                            else {
                                Ok(PriceUpdateRequest::new(
                                    DateTime::from_utc(dt.naive_utc(), chrono::Utc),
                                    args[1].trim().to_uppercase(),
                                    args[2].trim().to_uppercase(),
                                    args[3].trim().to_uppercase(),
                                    ff,
                                    bf,
                                ))
                            }
                        })
                })
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn validate_priceupdaterequest_with_valid_data() {
        let g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " UsD",
            " 1000.0 ",
            " 0.0009 ",
        ];
        let result = validate_price_update_input(&REQ, &g);

        let dt = DateTime::parse_from_rfc3339(REQ[0].trim()).unwrap();
        let ts = DateTime::from_utc(dt.naive_utc(), chrono::Utc);
        let expected = PriceUpdateRequest::new(
            ts,
            REQ[1].trim().to_uppercase(),
            REQ[2].trim().to_uppercase(),
            REQ[3].trim().to_uppercase(),
            REQ[4].trim().parse::<f32>().unwrap(),
            REQ[5].trim().parse::<f32>().unwrap(),
        );
        assert_eq!(expected, result.unwrap());
    }

    #[test]
    fn should_throw_invalid_argument_number_error() {
        let g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            // " BtC",
            " UsD",
            " 1000.0 ",
            " 0.0009 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::InvalidArgumentNumber
        );
    }

    #[test]
    fn should_throw_same_source_destination_currency_error() {
        let g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " BTC",
            " 1000.0 ",
            " 0.0009 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::SameSourceDestinationCurrency
        );
    }

    #[test]
    fn should_throw_invalid_timestamp_error() {
        let g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09: ssabc42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " Usd",
            " 1000.0 ",
            " 0.0009 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::InvalidTimestamp
        );
    }

    #[test]
    fn should_throw_invalid_forward_factor_error() {
        let  g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " Usd",
            " 1abc00 ",
            " 0.0009 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::InvalidForwardfactor
        );
    }

    #[test]
    fn should_throw_invalid_backward_factor_error() {
        let  g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " Usd",
            " 1000 ",
            " 0.xx0009 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::InvalidBackwardfactor
        );
    }

    #[test]
    fn should_throw_forward_backward_factor_multiply_error() {
        let g = Graph::new();
        const REQ: &[&str] = &[
            " 2017-11-01T09:42:23+00:00 ",
            " KrAKEN",
            " BtC",
            " Usd",
            " 1000 ",
            " 1.1 ",
        ];
        assert_eq!(
            validate_price_update_input(&REQ, &g).unwrap_err(),
            PriceUpdateRequestValidationError::ForwardBackwardFactorMultiplyError
        );
    }
}
