#!/bin/bash

echo "Stopping Lighthouse container..."

# Find and stop any container running the Lighthouse image
LIGHTHOUSE_CONTAINER=$(docker ps -q --filter ancestor=sigp/lighthouse)

if [ -n "$LIGHTHOUSE_CONTAINER" ]; then
  docker stop "$LIGHTHOUSE_CONTAINER"
  echo "Lighthouse container stopped."
else
  echo "No Lighthouse container is currently running."
fi
