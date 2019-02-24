# tenx-technical-exercise

The code is run and tested with Rust version 1.33.0-nightly on MacOS. Please run the following commands at source code directory.
* `$ cargo build`
* `$ cd target/debug`
* `$ ./tenx_technical_exercise`

![IMAGE](https://user-images.githubusercontent.com/156037/53299445-3307f780-3875-11e9-8286-47bc88ee3529.png)

I am not familiar with the algorithm mentioned, and I observed that the program went into endless loop in path finding for outlier case like below. At the moment I disply error message to user when such data is entered.

* 2017-11-01T09:42:23+00:00 KRAKEN BTC USD 1000.0 0.0009
* 2017-11-01T09:43:23+00:00 GDAX BTC USD 1001.0 0.0008
* 2017-11-01T09:42:23+00:00 BITTREX BTC USD 1200.0 0.0005 <- outlier case here
