#!/usr/bin/env python3
import unittest
import subprocess
import os
import shutil
import tempfile
import socket
import time
import threading

class TestNet(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.grotsky_bin = os.path.abspath("target/release/grotsky-rs")
        if not os.path.exists(self.grotsky_bin):
            self.grotsky_bin = os.path.abspath("build/grotsky-rs")

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_tcp_server(self):
        # Create a Grotsky server script
        script_path = os.path.join(self.temp_dir, "server.gr")
        with open(script_path, "w") as f:
            f.write("""
            let server = net.listenTcp(":0")
            # Print port so python client knows where to connect
            let addr = server.address()
            # Add explicit flush or just println (IO usually autoflushes on newline)
            io.println("PORT:" + addr) 
            
            let conn = server.accept()
            io.println("Accepted connection from: " + conn.address())
            
            let msg = conn.read()
            io.println("Received: " + msg)
            
            conn.write("Response from Grotsky")
            conn.close()
            server.close()
            """)

        env = os.environ.copy()
        env["LLVM_PROFILE_FILE"] = "grotsky-net-%p-%m.profraw"

        # Start server process
        proc = subprocess.Popen(
            [self.grotsky_bin, script_path],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env
        )

        try:
            # Read port line
            port = None
            while True:
                line = proc.stdout.readline()
                if not line:
                    break
                if "PORT:" in line:
                    # Address format might be "0.0.0.0:54321" or "[::]:54321"
                    # native.rs:650: socket.peer_addr()...to_string()
                    # native.rs:720: socket.local_addr()...to_string()
                    addr_str = line.strip().split("PORT:")[1]
                    # Parse port
                    port = int(addr_str.split(":")[-1])
                    break
            
            self.assertIsNotNone(port, "Could not get port from server")

            # Connect with python client
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.connect(("127.0.0.1", port))
            s.sendall(b"Hello from Python")
            response = s.recv(1024)
            s.close()

            self.assertEqual(response.decode(), "Response from Grotsky")
            
            # Wait for process to finish
            stdout, stderr = proc.communicate(timeout=5)
            self.assertEqual(proc.returncode, 0)
            self.assertIn("Received: Hello from Python", stdout)

        finally:
            if proc.poll() is None:
                proc.kill()

if __name__ == "__main__":
    unittest.main()

