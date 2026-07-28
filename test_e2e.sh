cargo run -p lightx-test > server.log 2>&1 &
SERVER_PID=$!
# Wait for the "LightX TLS Server listening securely" line
tail -f server.log | grep -m 1 "listening securely" -q
sleep 1
echo "Server is up! Testing..."
# Test CSS static route
echo "Testing /public/test.css"
curl -k -s -i https://127.0.0.1:8443/public/test.css > curl_css.log
# Test root route
echo "Testing /"
curl -k -s -i https://127.0.0.1:8443/ > curl_root.log
kill $SERVER_PID
echo "Done"
