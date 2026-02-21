#!/usr/bin/env python3
"""Simple HTTP server for testing countdown functionality."""

from http.server import HTTPServer, BaseHTTPRequestHandler
from datetime import datetime, timezone
import json
import time

# Global state for the countdown
start_time = time.time() + 15


class CountdownHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        global start_time

        now = time.time()

        # Reset if we've passed the start time
        if now >= start_time:
            start_time = now + 15

        # Convert to RFC3339 format
        dt = datetime.fromtimestamp(start_time, tz=timezone.utc)
        rfc3339 = dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        response = {
            "start_time": rfc3339
        }

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())

    def log_message(self, format, *args):
        print(f"[{time.strftime('%H:%M:%S')}] {args[0]}")


if __name__ == "__main__":
    port = 9847
    server = HTTPServer(("localhost", port), CountdownHandler)
    print(f"Server running on http://localhost:{port}")
    print(f"Initial start_time: {int(start_time)} (in 15 seconds)")
    server.serve_forever()
