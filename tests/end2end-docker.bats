#!/usr/bin/env bats

load "./bats-helpers/bats-support/load"
load "./bats-helpers/bats-assert/load"

setup_file() {
  echo "##################################" >&3
  echo "Setting up test environment..." >&3
  echo "   Pulling alpine/socat image" >&3
  docker pull alpine/socat

  echo "   Building and starting two omsn clients with a new network (mcastnet)" >&3
  docker compose -f tests/docker-compose-e2e.yml up -d

  echo "   Started, grace period before executing tests..." >&3
  sleep 5
  echo "Test environment ready." >&3
  echo "##################################\n" >&3
}

teardown_file() {
  echo "\n##################################" >&3
  echo "Tearing down the test environment..." >&3

  echo "   Removing receiver container..." >&3
  docker stop receiver
  docker rm receiver

  echo "   Stopping and removing composed container setup..." >&3
  docker compose -f tests/docker-compose-e2e.yml down -v

  echo "Test environment torn down." >&3
  echo "##################################" >&3
}

@test "multicast sender to receiver via zenoh" {

  # Starting receiver container
  docker run --detach --name receiver --net=tests_mcastnet alpine/socat -u UDP4-RECV:50000,bind=239.0.0.2,reuseaddr,ip-add-membership=239.0.0.2:0.0.0.0 STDOUT

  # Grace period
  sleep 2

  # Send datagram from sender container
  echo "hello-from-siteA" | docker run --rm --net=tests_mcastnet -i  -a stdin alpine/socat -u STDIN UDP4-DATAGRAM:239.0.0.1:50000

  # Grace period
  sleep 1

  # Get receiver logs
  run docker logs receiver

  echo $output

  # And make assertions
  assert_line --partial "hello-from-siteA"

}
