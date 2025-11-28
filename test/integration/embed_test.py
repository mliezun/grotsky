#!/usr/bin/env python3
import unittest
import subprocess
import os
import shutil
import tempfile

class TestEmbed(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.grotsky_bin = os.path.abspath("target/release/grotsky-rs")
        if not os.path.exists(self.grotsky_bin):
            self.grotsky_bin = os.path.abspath("build/grotsky-rs")

    def tearDown(self):
        shutil.rmtree(self.temp_dir)

    def test_embed(self):
        # 1. Create a simple script
        script_path = os.path.join(self.temp_dir, "hello.gr")
        with open(script_path, "w") as f:
            f.write('io.println("Hello from embedded!")')

        env = os.environ.copy()
        # Ensure profile file is unique for each process
        env["LLVM_PROFILE_FILE"] = "grotsky-embed-%p-%m.profraw"

        # 2. Compile it
        compile_cmd = [self.grotsky_bin, "compile", script_path]
        subprocess.check_call(compile_cmd, env=env)
        bytecode_path = script_path + "c"
        self.assertTrue(os.path.exists(bytecode_path))

        # 3. Embed it
        embed_cmd = [self.grotsky_bin, "embed", bytecode_path]
        subprocess.check_call(embed_cmd, env=env)
        
        exe_path = os.path.splitext(bytecode_path)[0] + ".exe"
        self.assertTrue(os.path.exists(exe_path))
        
        # 4. Run the embedded executable
        os.chmod(exe_path, 0o755)
        
        # Run it
        output = subprocess.check_output([exe_path], text=True, env=env)
        self.assertEqual(output.strip(), "Hello from embedded!")

if __name__ == "__main__":
    unittest.main()
