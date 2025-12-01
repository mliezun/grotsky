#!/usr/bin/env python3
"""
Integration test for grotsky-rs interpreter with blog project.
This test clones the blog repository and runs the site generation script.
"""

import unittest
import subprocess
import os
import shutil
import tempfile
import logging


logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)



class TestBlogIntegration(unittest.TestCase):
    """Integration test for blog project with grotsky-rs interpreter."""
    
    def setUp(self):
        """Set up test environment."""
        self.temp_dir = tempfile.mkdtemp(prefix="grotsky_test_")
        logger.info(f"Created temporary directory: {self.temp_dir}")
        
        self.blog_repo_path = os.path.join(self.temp_dir, "mliezun.github.io")
        
        # Check environment variable first
        if "GROTSKY_BINARY" in os.environ:
            self.grotsky_binary = os.environ["GROTSKY_BINARY"]
        else:
            # Fallback to build directory (where Makefile copies it)
            build_path = os.path.abspath(os.path.join(
                os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
                "build", "grotsky-rs"
            ))
            if os.path.exists(build_path):
                self.grotsky_binary = build_path
            else:
                # Fallback to target/release
                self.grotsky_binary = os.path.join(
                    os.path.dirname(os.path.dirname(os.path.dirname(__file__))),
                    "target", "release", "grotsky-rs"
                )
        
        logger.info(f"Blog repo will be cloned to: {self.blog_repo_path}")
        logger.info(f"Grotsky binary path: {self.grotsky_binary}")
        
        if not os.path.exists(self.grotsky_binary):
            raise FileNotFoundError(
                f"Grotsky binary not found at {self.grotsky_binary}. "
                "Please build the project first with 'cargo build --release'"
            )
    
    def tearDown(self):
        """Clean up test environment."""
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
            logger.info(f"Cleaned up temporary directory: {self.temp_dir}")
    
    def test_blog_site_generation(self):
        """Test that the blog site generation works with grotsky-rs."""
        logger.info("Starting blog site generation test")
        
        logger.info("Step 1: Cloning repository...")
        clone_result = self._clone_repository()
        self.assertEqual(clone_result.returncode, 0, 
                        f"Git clone failed with exit code {clone_result.returncode}")
        logger.info("Repository cloned successfully")
        
        logger.info("Step 2: Running site generation script...")
        generation_result = self._run_site_generation()
        self.assertEqual(generation_result.returncode, 0,
                        f"Site generation failed with exit code {generation_result.returncode}")
        logger.info("Site generation completed successfully")
        
        logger.info("All test steps completed successfully!")
    
    def _clone_repository(self):
        """Clone the blog repository."""
        repo_url = "https://github.com/mliezun/mliezun.github.io.git"
        logger.info(f"Cloning {repo_url} to {self.blog_repo_path}")
        
        try:
            result = subprocess.run(
                ["git", "clone", repo_url, self.blog_repo_path],
                capture_output=True,
                text=True,
                timeout=300
            )
            
            if result.stdout:
                logger.info(f"Clone stdout: {result.stdout}")
            if result.stderr:
                logger.warning(f"Clone stderr: {result.stderr}")
            
            return result
            
        except subprocess.TimeoutExpired:
            logger.error("Git clone timed out after 5 minutes")
            raise
        except FileNotFoundError:
            logger.error("Git command not found. Please ensure git is installed.")
            raise
    
    def _run_site_generation(self):
        """Run the site generation script with grotsky-rs."""
        script_path = os.path.join(self.blog_repo_path, "src", "generate_site.gr")
        
        if not os.path.exists(script_path):
            raise FileNotFoundError(f"Site generation script not found at {script_path}")
        
        logger.info(f"Running: {self.grotsky_binary} {script_path}")
        logger.info(f"Working directory: {self.blog_repo_path}")
        
        env = os.environ.copy()
        env["RUST_BACKTRACE"] = "full"
        env["GROTKSY_DEBUG"] = "1"
        # Use absolute path for profile file to ensure it's written to project root
        # instead of the temporary directory which gets deleted
        env["LLVM_PROFILE_FILE"] = os.path.abspath("grotsky-%p-%m.profraw")
        
        try:
            result = subprocess.run(
                [self.grotsky_binary, script_path],
                cwd=self.blog_repo_path,
                capture_output=True,
                text=True,
                env=env,
                timeout=600
            )
            
            if result.stdout:
                logger.info(f"Generation stdout: {result.stdout}")
            if result.stderr:
                logger.warning(f"Generation stderr: {result.stderr}")
            
            return result
            
        except subprocess.TimeoutExpired:
            logger.error("Site generation timed out after 10 minutes")
            raise
        except FileNotFoundError:
            logger.error(f"Grotsky binary not found at {self.grotsky_binary}")
            raise


if __name__ == "__main__":
    unittest.main(verbosity=2)
