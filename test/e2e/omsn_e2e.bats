#!/usr/bin/env bats

@test "omsn clients exchange datagrams via zenoh" {
  SITE1="siteA"
  APP1="appA"
  SITE2="siteB"
  APP2="appB"
  SIMID="simtest"
  GROUP1="239.0.0.1"
  GROUP2="239.0.0.2"
  PORT="50000"

  # Start receiver in background, capture output
  RECEIVER_OUT=$(mktemp)
  ./target/debug/omsn --simulation-id "$SIMID" --site-id "$SITE1" --application-id "$APP1" --group "$GROUP1" --port "$PORT" --stats --verbose > "$RECEIVER_OUT" 2>&1 &
  RECEIVER_PID=$!
  sleep 2

  # Start sender in background, capture output
  SENDER_OUT=$(mktemp)
  ./target/debug/omsn --simulation-id "$SIMID" --site-id "$SITE2" --application-id "$APP2" --group "$GROUP2" --port "$PORT" --stats --verbose > "$SENDER_OUT" 2>&1 &
  SENDER_PID=$!
  sleep 2

  # Send a UDP datagram to the receiver's multicast group
  echo -n "hello-from-$SITE2" | socat - UDP4-DATAGRAM:$GROUP1:$PORT,sp=60000
  sleep 5

  # Kill clients
  kill $RECEIVER_PID $SENDER_PID 2>/dev/null || true
  sleep 1

  # Check receiver output for datagram receipt and stats
  grep "hello-from-$SITE2" "$RECEIVER_OUT"
  grep "From SITE_ID: $SITE2 | APPLICATION_ID: $APP2 =>" "$RECEIVER_OUT"

  # Cleanup
  rm -f "$RECEIVER_OUT" "$SENDER_OUT"
}
