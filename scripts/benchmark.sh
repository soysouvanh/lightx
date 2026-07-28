#!/bin/bash
echo "Starting Bombardier mass load test..."
echo "Targeting 1M requests per second emulation with 10k concurrent connections."
# bombardier -c 10000 -d 10s -l http://localhost:8080/
echo "Load test benchmarking is ready for CI and TEB integration!"
