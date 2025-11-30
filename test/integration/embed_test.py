#!/usr/bin/env python3
import unittest
import subprocess
import os
import shutil
import tempfile
import platform

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
        
        # On macOS, handle Gatekeeper issues with unsigned executables
        if platform.system() == "Darwin":
            try:
                # Remove quarantine attribute if it exists
                result = subprocess.run(["xattr", "-l", exe_path], 
                                      capture_output=True, text=True)
                if "com.apple.quarantine" in result.stdout:
                    subprocess.check_call(["xattr", "-d", "com.apple.quarantine", exe_path], 
                                        stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
                # Try ad-hoc code signing to bypass Gatekeeper
                subprocess.check_call(["codesign", "--force", "--sign", "-", exe_path], 
                                    stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
            except (subprocess.CalledProcessError, FileNotFoundError):
                # Commands failed, continue anyway - might work or might fail
                pass
        
        # Run it
        try:
            result = subprocess.run([exe_path], capture_output=True, text=True, env=env, timeout=5)
            if result.returncode != 0:
                # On macOS, SIGKILL often means Gatekeeper blocked execution
                if platform.system() == "Darwin" and result.returncode == -9:
                    self.skipTest("Embedded executable blocked by macOS Gatekeeper (SIGKILL). "
                                "This is expected for unsigned executables on macOS.")
                self.fail(f"Executable failed with return code {result.returncode}. "
                         f"stderr: {result.stderr}")
            output = result.stdout
            self.assertEqual(output.strip(), "Hello from embedded!")
        except subprocess.TimeoutExpired:
            self.fail("Executable timed out")
        except subprocess.CalledProcessError as e:
            # Handle SIGKILL on macOS
            if platform.system() == "Darwin" and hasattr(e, 'returncode') and e.returncode == -9:
                self.skipTest("Embedded executable blocked by macOS Gatekeeper (SIGKILL). "
                            "This is expected for unsigned executables on macOS.")
            raise

if __name__ == "__main__":
    unittest.main()
