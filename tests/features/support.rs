use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMPORARY_SITE_NUMBER: AtomicUsize = AtomicUsize::new(0);

pub struct TemporarySite {
    pub posts_directory: PathBuf,
    pub output_directory: PathBuf,
    root_directory: PathBuf,
}

impl TemporarySite {
    pub fn create(test_name: &str) -> Self {
        let temporary_site_number = NEXT_TEMPORARY_SITE_NUMBER.fetch_add(1, Ordering::Relaxed);
        let root_directory = std::env::temp_dir().join(format!(
            "blog-feature-{test_name}-{}-{temporary_site_number}",
            std::process::id()
        ));
        let posts_directory = root_directory.join("posts");
        let output_directory = root_directory.join("dist");
        let assets_directory = root_directory.join("assets");
        fs::create_dir_all(&posts_directory)
            .expect("the temporary post source directory should be created");
        fs::create_dir_all(&assets_directory)
            .expect("the temporary assets directory should be created");

        let project_assets_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        for asset_filename in ["blog.css", "index.html"] {
            let source_asset_path = project_assets_directory.join(asset_filename);
            let temporary_asset_path = assets_directory.join(asset_filename);
            fs::copy(&source_asset_path, &temporary_asset_path)
                .expect("project assets should be copied to the temporary site");
        }

        Self {
            posts_directory,
            output_directory,
            root_directory,
        }
    }

    pub fn write_post(&self, relative_post_path: &str, post_contents: &str) {
        let post_path = self.posts_directory.join(relative_post_path);
        let post_directory = post_path
            .parent()
            .expect("the post path should have a parent directory");
        fs::create_dir_all(post_directory).expect("the post directory should be created");
        fs::write(post_path, post_contents).expect("the post fixture should be written");
    }

    pub fn run_generator(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_blog"))
            .env("SOURCE", &self.posts_directory)
            .env("OUTPUT", &self.output_directory)
            .env("SITE_URL", "https://ksevelyar.rusty-cluster.net")
            .output()
            .expect("the site generator process should start")
    }

    pub fn read_generated_file(&self, relative_file_path: &str) -> String {
        fs::read_to_string(self.output_directory.join(relative_file_path))
            .expect("the generated site file should be readable")
    }

    pub fn generated_file_exists(&self, relative_file_path: &str) -> bool {
        self.output_directory.join(relative_file_path).is_file()
    }
}

impl Drop for TemporarySite {
    fn drop(&mut self) {
        if let Err(cleanup_error) = fs::remove_dir_all(&self.root_directory) {
            eprintln!(
                "failed to remove temporary site directory {}: {cleanup_error}",
                self.root_directory.display()
            );
        }
    }
}
