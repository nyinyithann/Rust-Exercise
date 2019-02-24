extern crate quick_error;

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum GraphError{
        PathNotFound{
            description("No path found at the moment. System doesn't have enough data to provide answer to your request.")
        }
        InvalidPath{
            description("Invalid request. Source exchange and currency should not be the same as destination's")
        }
        Critical{
            description("There is an a critical error occured inside the system. Please wipe out all the existing data and continue using the system.")
        }
    }
}

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum PriceUpdateRequestValidationError{
        InvalidArgumentNumber{
            description("Invalid request : the number of price-update-request arguments must be 6")
        }
        SameSourceDestinationCurrency{
            description("The currency of Source should not be the same as that of Destination")
        }
        InvalidTimestamp{
            description("Invalid timestamp")
        }
        InvalidForwardfactor{
            description("Invalid forward factor")
        }
        InvalidBackwardfactor{
            description("Invalid backward factor")
        }
        ForwardBackwardFactorMultiplyError{
            description("The product of forward factor and backward factor should be less than or equal to 1")
        }
        CrossForwardBackwardFactorMultiplyError{
            description("Invalid input based on the current algorithm of the system. The algorithm works only when the product of forward and backward factor of each path is less than or equal to 1")
        }
    }
}

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum ExchangeRateRequestValidationError{
        InvalidArgumentNumber{
            description("Invalid request : the number of exchange-rate-request arguments must be 4")
        }
    }
}
