# nats-jetstream-example
example using NATS jetstream


## Setup 

You can setup the cluster running `docker compose up`

debug and manually test by launching another container into the same docker network with `docker run --network nats --rm -it natsio/nats-box`

from there you can subscribe to a channel with `nats sub -s nats://nats:4222 channel` and send test messages with `nats pub -s "nats://nats-1:4222" channel message`

